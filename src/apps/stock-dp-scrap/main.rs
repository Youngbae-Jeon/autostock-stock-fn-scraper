use chrono::{Duration, Local, NaiveDate};

use stock_fn_scraper::{data_source, logger};
use stock_fn_scraper::entities::{EntityDao, RepoTx, Repository, Stock, StockPrice};
use stock_fn_scraper::repository::{self, DatabaseConfig, Repo};
use stock_fn_scraper::types::Error;


#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let today = Local::now().date_naive();
	let stocks = repo.stocks().list().await.unwrap();

	for stock in stocks.iter() {
		if let Err(e) = work_with_stock(&repo, stock, today).await {
			log::error!("Error: {} (Stock `{}|{}`)", e.message, stock.code, stock.name);
		}

		break; // stop for debug
	}
}

const BEGIN_DATE_LIMIT: NaiveDate = NaiveDate::from_ymd_opt(1990, 1, 3).unwrap();

async fn work_with_stock(repo: &Repo, stock: &Stock, base_date: NaiveDate) -> Result<(), Error> {
	let mut cached = repo.stock_prices().oldest_and_latest(&stock.code).await?;
	if let Some((oldest, latest)) = &cached {
		log::debug!("Cached: `{}|{}` oldest={} latest={}", stock.code, stock.name, oldest.ord_date, latest.ord_date);

		if base_date == latest.ord_date {
			// 기준일자 == 캐시종료일이면 캐시가 유효한 것으로 본다
			log::debug!("`{}|{}` 기준일자 == 캐시종료일이므로 캐시가 유효한 것으로 본다 --> skip ({})", stock.code, stock.name, base_date.format("%Y-%m-%d"));
			return Ok(());
		}

		if base_date > latest.ord_date {
			// 기준일자 < 캐시의 최종일자이면 캐시의 무결함이 의심되므로 캐시를 무효화하고 계속 진행한다
			log::debug!("`{}|{}` DELETE prices because cached may be invalid", stock.code, stock.name);
			repo.stock_prices().delete_all(&stock.code).await?;
			cached = None;
		}
	}

	let last_date_of_cached = cached.as_ref().map(|(_, latest)| latest.ord_date)
		.or(stock.list_date)
		.unwrap_or(BEGIN_DATE_LIMIT);
	let prices_after_cached = StockPriceFetcher::fetch(stock, last_date_of_cached, base_date).await?;
	let updates_for_cached = if cached.as_ref().is_some_and(|c| cached_prices_are_valid(c, &prices_after_cached)) {
		let list_date = stock.list_date.unwrap_or(BEGIN_DATE_LIMIT);
		let prices = StockPriceFetcher::fetch(stock, list_date, last_date_of_cached).await?;
		Some(prices)
	} else {
		None
	};

	update_stock_prices_cache(repo, stock, &prices_after_cached, updates_for_cached.as_deref()).await
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

	if let Some(updates) = updates_for_cached {
		tx.stock_prices().delete_all(&stock.code).await?;
		tx.stock_prices().insert_all(&stock.code, updates).await?;
	}

	tx.stocks().find_for_update(&stock.code).await?
		.ok_or_else(|| format!("Not Found Stock Code {}", stock.code))?;
	tx.stock_prices().insert_all(&stock.code, &prices_after_cached).await?;
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
			cumul.insert(0, list);
			end = start;
		}

		let result: Vec<StockPrice> = cumul.into_iter().flatten().collect();
		Ok(result)
	}
}
