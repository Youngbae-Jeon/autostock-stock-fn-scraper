use async_trait::async_trait;

use crate::{entities::{FiAnnualsDao, FiQuartersDao, StockPricesDao, StocksDao}, types::Error};

pub trait EntityDao: Send + Sync {
	fn stocks(&self) -> &(dyn StocksDao + Sync);
	fn stock_prices(&self) -> &(dyn StockPricesDao + Sync);
	fn fi_annuals(&self) -> &(dyn FiAnnualsDao + Sync);
	fn fi_quarters(&self) -> &(dyn FiQuartersDao + Sync);
}

#[async_trait]
pub trait Repository: EntityDao + AsRef<dyn EntityDao> + Clone {
	async fn transaction(&self) -> Result<impl RepoTx, Error>;
	async fn test_connection(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait RepoTx: EntityDao + AsRef<dyn EntityDao> {
	async fn commit(self) -> Result<(), Error>;
	async fn rollback(self) -> Result<(), Error>;
}
