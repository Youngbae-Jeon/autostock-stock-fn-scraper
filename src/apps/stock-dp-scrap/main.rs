use chrono::{Datelike, Duration, Local, NaiveDate};

use stock_fn_scraper::{data_source, logger};
use stock_fn_scraper::entities::{EntityDao, RepoTx, Repository, Stock, StockPrice};
use stock_fn_scraper::repository::{self, DatabaseConfig, Repo};
use stock_fn_scraper::types::Error;


#[tokio::main]
async fn main() {
	#[cfg(feature = "dotenv")]
	dotenvy::dotenv().ok();

	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let today = Local::now().date_naive();
	let stocks = repo.stocks().list().await.unwrap();

	for stock in stocks.iter().filter(|s| s.code == "069500") {
		if let Err(e) = work_with_stock(&repo, stock, today).await {
			log::error!("Error: {} (Stock `{}|{}`)", e.message, stock.code, stock.name);
		}

		// break; // stop for debug
	}
}

const BEGIN_DATE_LIMIT: NaiveDate = NaiveDate::from_ymd_opt(1990, 1, 3).unwrap();

async fn work_with_stock(repo: &Repo, stock: &Stock, today: NaiveDate) -> Result<(), Error> {
	let mut cached = repo.stock_prices().oldest_and_latest(&stock.code).await?;
	if let Some((oldest, latest)) = &cached {
		log::debug!("Cached: `{}|{}` oldest={} latest={}", stock.code, stock.name, oldest.ord_date, latest.ord_date);

		let lbd = last_business_day(today);
		println!("lbd={} today={}", lbd.format("%Y-%m-%d"), today.format("%Y-%m-%d"));
		if lbd == latest.ord_date {
			log::debug!("`{}|{}` 직전영업일 == 캐시종료일이므로 캐시가 유효한 것으로 판단 --> skip", stock.code, stock.name);
			return Ok(());
		}

		if lbd < latest.ord_date {
			log::info!("`{}|{}` 직전영업일 < 캐시종료일이므로 캐시의 무결함이 의심됨 -> 캐시 삭제", stock.code, stock.name);
			repo.stock_prices().delete_all(&stock.code).await?;
			cached = None;
		}
	}

	let last_date_of_cached = cached.as_ref().map(|(_, latest)| latest.ord_date)
		.or(stock.list_date)
		.unwrap_or(BEGIN_DATE_LIMIT);
	let mut prices_after_cached = StockPriceFetcher::fetch(stock, last_date_of_cached, today).await?;
	if prices_after_cached.is_empty() {
		log::warn!("`{}|{}` 가격 데이터 가져오기 실패 -> 상장 해지 종목 의심", stock.code, stock.name);
		return Ok(());
	}

	if cached.as_ref().is_some_and(|c| !cached_prices_are_valid(c, &prices_after_cached)) {
		let list_date = stock.list_date.unwrap_or(BEGIN_DATE_LIMIT);
		let cache_replacement = StockPriceFetcher::fetch(stock, list_date, last_date_of_cached).await?;

		// set diff value of first day of new part(prices_after_cached)
		if let Some(first_of_new_part) = prices_after_cached.first_mut() {
			set_stock_price_diff_from(first_of_new_part, cache_replacement.last());
		}

		update_stock_prices_cache(repo, stock, &prices_after_cached, Some(&cache_replacement)).await

	} else {
		if cached.is_some() {
			prices_after_cached = prices_after_cached.into_iter().filter(|p| p.ord_date > last_date_of_cached).collect();
		}
		update_stock_prices_cache(repo, stock, &prices_after_cached, None).await
	}
}

fn last_business_day(mut date: NaiveDate) -> NaiveDate {
	loop {
		date -= Duration::days(1);
		let w = date.weekday();
		if w != chrono::Weekday::Sat && w != chrono::Weekday::Sun {
			break date;
		}
	}
}

fn cached_prices_are_valid((_, cached_latest): &(StockPrice, StockPrice), fresh_fetched: &[StockPrice]) -> bool {
	if let Some(p) = fresh_fetched.iter().find(|p| p.ord_date == cached_latest.ord_date) {
		p.closing == cached_latest.closing
	} else {
		true
	}
}

async fn update_stock_prices_cache(repo: &Repo, stock: &Stock, prices_after_cached: &[StockPrice], updates_for_cached: Option<&[StockPrice]>) -> Result<(), Error> {
	let tx = repo.transaction().await?;

	tx.stocks().find_for_update(&stock.code).await?
		.ok_or_else(|| format!("Not Found Stock Code {}", stock.code))?;

	if let Some(updates) = updates_for_cached {
		tx.stock_prices().delete_all(&stock.code).await?;

		if !updates.is_empty() {
			tx.stock_prices().insert_all(&stock.code, updates).await?;
		}
	}

	if !prices_after_cached.is_empty() {
		tx.stock_prices().insert_all(&stock.code, &prices_after_cached).await?;
	}

	tx.commit().await?;
	Ok(())
}

struct StockPriceFetcher<'a> {
	stock: &'a Stock,
	start: NaiveDate,
	end: NaiveDate,
}

impl<'a> StockPriceFetcher<'a> {
	async fn fetch(stock: &'a Stock, start: NaiveDate, end: NaiveDate) -> Result<Vec<StockPrice>, Error> {
		let fetcher = Self { stock, start, end };
		fetcher.fetch_inner().await
	}

	async fn fetch_inner(&self) -> Result<Vec<StockPrice>, Error> {
		let mut cumul: Vec<Vec<StockPrice>> = Vec::new();
		let mut end = self.end;
		while self.start < end {
			let start = (end - Duration::days(150)).max(self.start);
			let list = data_source::query_stock_prices(&self.stock.code, start, end).await?;
			if list.is_empty() {
				break;
			}

			cumul.insert(0, list);
			end = start;
		}

		let mut result: Vec<StockPrice> = cumul.into_iter().flatten().collect();
		Self::set_diff_values(&mut result);
		Ok(result)
	}

	fn set_diff_values(prices: &mut [StockPrice]) {
		let mut last_closing = 0;
		for price in prices {
			if price.closing != 0 {
				if last_closing != 0 {
					price.diff = price.closing as i32 - last_closing as i32;
				}
			}
			last_closing = price.closing;
		}
	}
}

fn set_stock_price_diff_from(sp: &mut StockPrice, last_sp: Option<&StockPrice>) {
	if sp.closing == 0 {
		let closing_val_of_last = last_sp.map(|p| p.closing).unwrap_or(0);
		sp.diff = sp.closing as i32 - closing_val_of_last as i32;
	}
}
