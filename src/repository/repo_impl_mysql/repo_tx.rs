use async_trait::async_trait;
use mysql_async::Transaction;
use repo_helper::mysql::QueryObject;
use tokio::sync::Mutex;

use crate::types::Error;
use crate::entities::{EntityDao, FiAnnualsDao, FiQuartersDao, RepoTx, StockPricesDao, StocksDao};

pub struct RepoTxImpl {
	native_tx: Mutex<Transaction<'static>>,
}

impl RepoTxImpl {
	pub fn new(mysql_tx: Transaction<'static>) -> Self {
		RepoTxImpl { native_tx: Mutex::new(mysql_tx) }
	}

	pub async fn get_query_object(&self) -> Result<QueryObject<'_>, Error> {
		let native = self.native_tx.lock().await;
		Ok(QueryObject::Tx(native))
	}
}

impl EntityDao for RepoTxImpl {
	fn stocks(&self) -> &(dyn StocksDao + Sync) {
		self
	}
	fn stock_prices(&self) -> &(dyn StockPricesDao + Sync) {
		self
	}
	fn fi_annuals(&self) -> &(dyn FiAnnualsDao + Sync) {
		self
	}
	fn fi_quarters(&self) -> &(dyn FiQuartersDao + Sync) {
		self
	}
}

impl AsRef<dyn EntityDao> for RepoTxImpl {
	fn as_ref(&self) -> &(dyn EntityDao + 'static) {
		self
	}
}

#[async_trait]
impl RepoTx for RepoTxImpl {
	async fn commit(self) -> Result<(), Error> {
		self.native_tx.into_inner().commit().await
			.map_err(Error::from)
	}
	async fn rollback(self) -> Result<(), Error> {
		self.native_tx.into_inner().rollback().await
			.map_err(Error::from)
	}
}
