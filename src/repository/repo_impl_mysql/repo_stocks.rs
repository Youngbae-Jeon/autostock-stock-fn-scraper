use async_trait::async_trait;
use chrono::NaiveDate;
use mysql_async::prelude::FromRow;
use repo_helper::{Filter, SqlFilter, database_table, mysql::{MySqlHelper, QueryObject}};

use crate::types::{Error, Market};
use crate::entities::{Stock, StocksDao};
use crate::repository::repo_impl_mysql::{convert::IntoRepoResult, repo::RepoImpl, repo_tx::RepoTxImpl};


#[async_trait]
impl StocksDao for RepoImpl {
	async fn list(&self) -> Result<Vec<Stock>, Error> {
		let mut q = self.get_query_object().await?;
		list(&mut q).await
	}
}

#[async_trait]
impl StocksDao for RepoTxImpl {
	async fn list(&self) -> Result<Vec<Stock>, Error> {
		let mut q = self.get_query_object().await?;
		list(&mut q).await
	}
}


database_table! {
	#[table_name = "item_info", derive(FromRow)]
	EntityRow {
		code: String,
		info_date: NaiveDate,
		name: String,
		market: String,
		std_code: Option<String>,
		list_date: Option<NaiveDate>,
		kind: Option<String>,
		secu_group: Option<String>,
		sect: Option<String>,
		par: Option<u32>,
		list_shares: Option<u64>,
	}
}
impl TryFrom<EntityRow> for Stock {
	type Error = Error;

	fn try_from(value: EntityRow) -> Result<Self, Self::Error> {
		Ok(Self {
			code: value.code,
			info_date: value.info_date,
			name: value.name,
			market: value.market.as_str().try_into()?,
			std_code: value.std_code,
			list_date: value.list_date,
			kind: value.kind,
			secu_group: value.secu_group,
			sect: value.sect,
			par: value.par,
			list_shares: value.list_shares,
		})
	}
}


const TABLE: &str = EntityRow::TABLE_NAME;
const FIELDS: &str = EntityRow::TABLE_FIELDS;

async fn list(q: &mut QueryObject<'_>) -> Result<Vec<Stock>, Error> {
	let key = SqlFilter::default()
		.with("market", &Filter::In(vec!["KOSPI", "KOSDAQ"]));
	let key_clause = key.with_named_binding_holder();
	let sql = format!("SELECT {FIELDS} FROM {TABLE} WHERE {key_clause} ORDER BY name");
	log::debug!("{} -- {}", sql, key);

	let stmt = q.prep(sql).await?;
	let rows: Vec<EntityRow> = q.exec(&stmt, key.params()).await?;
	rows.into_repo_result()
}
