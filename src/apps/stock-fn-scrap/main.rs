use chrono::Local;

use stock_fn_scraper::data_source;
use stock_fn_scraper::logger;
use stock_fn_scraper::entities::EntityDao;
use stock_fn_scraper::repository::{self, DatabaseConfig};


#[tokio::main]
async fn main() {
	#[cfg(feature = "dotenv")]
	dotenvy::dotenv().ok();

	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let today = Local::now().date_naive();
	let stocks = repo.stocks().list().await.unwrap();
	let stocks_len = stocks.len();

	let active_stocks = stocks.into_iter().fold(Vec::new(), |mut acc, stock| {
		if today - stock.info_date > chrono::Duration::days(10) {
			log::info!("Stock `{}|{}` Outdated and skipped (date:{})", stock.code, stock.name, stock.info_date);
			return acc;
		}
		acc.push(stock);
		acc
	});

	let active_stocks_len = active_stocks.len();
	let skipped_cnt = stocks_len - active_stocks_len;
	let mut fetched_cnt = 0_usize;
	let mut error_cnt = 0_usize;
	let now = Local::now();

	for stock in active_stocks.iter() {
		match data_source::query_stock_financials(&stock.code).await {
			Ok(financials) => {
				log::info!("Financials of Stock `{}|{}` fetched. ({} annuals, {} quarters)", stock.code, stock.name, financials.annuals.list.len(), financials.quarters.list.len());
				financials.save(&repo).await.unwrap();
				fetched_cnt += 1;
			},
			Err(err) => {
				log::error!("Error: {:?} - Stock: `{}|{}`", err, stock.code, stock.name);
				error_cnt += 1;
			}
		}

		let delay = ((Local::now() - now).num_milliseconds() as f32) / 100 as f32;
		log::info!("{fetched_cnt}/{active_stocks_len} fetched. ({skipped_cnt} skipped, {error_cnt} errors) - {delay:.1} delayed");
	}
}
