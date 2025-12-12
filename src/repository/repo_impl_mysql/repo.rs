use async_trait::async_trait;
use mysql_async::{Pool, Opts, PoolOpts, PoolConstraints, OptsBuilder, prelude::Queryable, TxOpts};
use repo_helper::mysql::QueryObject;

use crate::types::Error;
use crate::repository::repo_impl_mysql::repo_tx::RepoTxImpl;
use crate::entities::{EntityDao, FiAnnualsDao, FiQuartersDao, RepoTx, Repository, StocksDao};

#[derive(Clone)]
pub struct RepoImpl {
	pool: Pool,
}

impl RepoImpl {
	pub fn from(pool: Pool) -> RepoImpl {
		RepoImpl { pool }
	}

	pub async fn new(dburl: &str, max_connections: usize) -> RepoImpl {
		let opts = Opts::from_url(dburl)
			.unwrap_or_else(|_| {
				log::error!("invalid database url: {}", dburl);
				panic!("invalid database url: {}", dburl);
			});
		let pool_opts = PoolOpts::default()
			.with_constraints(PoolConstraints::new(1, max_connections).unwrap());
		let opts = OptsBuilder::from_opts(opts)
			.client_found_rows(true)
			.pool_opts(pool_opts);
		RepoImpl::from(Pool::new(opts))
	}

	pub async fn get_query_object(&self) -> Result<QueryObject<'_>, Error> {
		let conn = self.pool.get_conn().await?;
		Ok(QueryObject::Conn(conn))
	}
}

impl EntityDao for RepoImpl {
	fn stocks(&self) -> &(dyn StocksDao + Sync) {
		self
	}
	fn fi_annuals(&self) -> &(dyn FiAnnualsDao + Sync) {
		self
	}
	fn fi_quarters(&self) -> &(dyn FiQuartersDao + Sync) {
		self
	}
}

impl AsRef<dyn EntityDao> for RepoImpl {
	fn as_ref(&self) -> &(dyn EntityDao + 'static) {
		self
	}
}

#[async_trait]
impl Repository for RepoImpl {
	async fn transaction(&self) -> Result<impl RepoTx, Error> {
		let natx = self.pool
			.start_transaction(TxOpts::default()).await?;
		Ok(RepoTxImpl::new(natx))
	}

	async fn test_connection(&self) -> Result<(), Error> {
		let mut conn = self.pool.get_conn().await?;
		let _ = conn.query_drop("SELECT 1").await?;
		Ok(())
	}
}
