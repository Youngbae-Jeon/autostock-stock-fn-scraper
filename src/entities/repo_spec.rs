use async_trait::async_trait;

use crate::{entities::StocksDao, types::Error};

pub trait EntityDao: Send + Sync {
	fn stocks(&self) -> &(dyn StocksDao + Sync);
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
