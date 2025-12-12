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

	for stock in stocks.iter().filter(|s| s.code == "029780") {
		log::info!("Stock: {} {} {}", stock.code, stock.info_date, stock.name);
		if today - stock.info_date > chrono::Duration::days(10) {
			log::info!("Outdated (skip)");
		}

		match query_stock_financials(&stock.code).await {
			Ok(financials) => {
				println!("Annuals: {:?}", financials.annuals.list);
				println!("Quarters: {:?}", financials.quarters.list);
				if let Err(err) = financials.save(&repo).await {
					log::error!("Error saving financials: {:?}", err);
				}
			},
			Err(err) => {
				log::error!("Error: {:?}", err);
			}
		}
	}
}
