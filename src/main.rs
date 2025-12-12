use crate::{entities::EntityDao, repository::DatabaseConfig};

mod logger;
mod types;
mod entities;
mod repository;

#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let stocks = repo.stocks().list().await.unwrap();
	for stock in stocks.iter() {
		println!("Stock: {} {} {}", stock.code, stock.info_date, stock.name);
	}
}
