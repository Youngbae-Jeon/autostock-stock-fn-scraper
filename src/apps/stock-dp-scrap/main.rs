use chrono::{Duration, Local, NaiveDate};

use stock_fn_scraper::logger;
use stock_fn_scraper::entities::{EntityDao, Stock};
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

		if let Err(e) = fetch_stock_prices(&repo, stock, today).await {
			log::error!("Error: {} (Stock `{}|{}`)", e.message, stock.code, stock.name);
		}
	}
}

async fn fetch_stock_prices(repo: &Repo, stock: &Stock, base_date: NaiveDate) -> Result<(), Error> {
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
			let the_next_of_latest = latest.ord_date + Duration::days(1);
			log::debug!("`{}|{}` DELETE prices BEFORE {}", stock.code, stock.name, the_next_of_latest.format("%Y-%m-%d"));
			repo.stock_prices().delete_before(&stock.code, the_next_of_latest).await?;
			cached = None;
		}
	}

	// TODO: something
	Ok(())
}
