use std::process::exit;

use chrono::Local;

use crate::{data_source::{query_stock_financials}, entities::EntityDao, repository::{DatabaseConfig, Repo}, types::Error};

mod logger;
mod types;
mod entities;
mod repository;
mod data_source;
mod fi_registry;


#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let today = Local::now().date_naive();
	let stocks = repo.stocks().list().await.unwrap();

	let stocks_len = stocks.len();
	let mut fetched_stocks = 0;
	let mut skipped_stocks = 0;
	let mut error_stocks = 0;
	let now = Local::now();

	for stock in stocks.iter() {
		if today - stock.info_date > chrono::Duration::days(10) {
			log::info!("Stock `{}|{}` Outdated and skipped (date:{})", stock.code, stock.name, stock.info_date);
			skipped_stocks += 1;

		} else {
			match query_stock_financials(&stock.code).await {
				Ok(financials) => {
					log::info!("Financials of Stock `{}|{}` fetched. ({} annuals, {} quarters)", stock.code, stock.name, financials.annuals.list.len(), financials.quarters.list.len());
					financials.save(&repo).await.unwrap();
					fetched_stocks += 1;
				},
				Err(err) => {
					log::error!("Error: {:?} - Stock: `{}|{}`", err, stock.code, stock.name);
					error_stocks += 1;
				}
			}
		}

		let delay = ((Local::now() - now).num_milliseconds() as f32) / 100 as f32;
		log::info!("{fetched_stocks}/{stocks_len} fetched. ({skipped_stocks} skipped, {error_stocks} errors) - {delay:.1} delayed");
	}
}
