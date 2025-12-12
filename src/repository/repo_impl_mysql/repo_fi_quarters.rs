use async_trait::async_trait;
use mysql_async::{params, prelude::FromRow};
use repo_helper::{SqlValues, database_table, mysql::{MySqlHelper, QueryObject}};

use crate::{entities::{FiQuarterData, FiQuartersDao, FinancialInfo}, types::{Error, YearMonth}};
use crate::repository::repo_impl_mysql::{convert::IntoRepoResult, repo::RepoImpl, repo_tx::RepoTxImpl};


#[async_trait]
impl FiQuartersDao for RepoImpl {
	async fn find(&self, stock_code: &str, year: u16, month: u8) -> Result<Option<FinancialInfo>, Error> {
		let mut q = self.get_query_object().await?;
		find(&mut q, stock_code, year, month).await
	}
	async fn list(&self, stock_code: &str) -> Result<Vec<FinancialInfo>, Error> {
		let mut q = self.get_query_object().await?;
		list(&mut q, stock_code).await
	}
	async fn insert(&self, annual: &FinancialInfo) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		insert(&mut q, annual).await
	}
	async fn update(&self, annual: &mut FinancialInfo, data: FiQuarterData) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		update(&mut q, annual, data).await
	}
}

#[async_trait]
impl FiQuartersDao for RepoTxImpl {
	async fn find(&self, stock_code: &str, year: u16, month: u8) -> Result<Option<FinancialInfo>, Error> {
		let mut q = self.get_query_object().await?;
		find(&mut q, stock_code, year, month).await
	}
	async fn list(&self, stock_code: &str) -> Result<Vec<FinancialInfo>, Error> {
		let mut q = self.get_query_object().await?;
		list(&mut q, stock_code).await
	}
	async fn insert(&self, annual: &FinancialInfo) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		insert(&mut q, annual).await
	}
	async fn update(&self, annual: &mut FinancialInfo, data: FiQuarterData) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		update(&mut q, annual, data).await
	}
}


database_table! {
	#[table_name = "fi_quarters", derive(FromRow)]
	EntityRow {
		stock_code: String,
		year: u16,
		month: u8,
		sales: Option<f32>,
		profit: Option<f32>,
		net_income: Option<f32>,
		dividend: Option<f32>,
		dividend_yield: Option<f32>,
	}
}
impl TryFrom<EntityRow> for FinancialInfo {
	type Error = Error;

	fn try_from(value: EntityRow) -> Result<Self, Self::Error> {
		Ok(Self {
			stock_code: value.stock_code,
			year_month: YearMonth::new(value.year, value.month),
			sales: value.sales,
			profit: value.profit,
			net_income: value.net_income,
			dividend: value.dividend,
			dividend_yield: value.dividend_yield,
		})
	}
}


const TABLE: &str = EntityRow::TABLE_NAME;
const FIELDS: &str = EntityRow::TABLE_FIELDS;

async fn find(q: &mut QueryObject<'_>, stock_code: &str, year: u16, month: u8) -> Result<Option<FinancialInfo>, Error> {
	let sql = format!("SELECT {FIELDS} FROM {TABLE} WHERE stock_code=:stock_code AND year=:year AND month=:month");
	log::debug!("{sql} -- stock_code={stock_code}, year={year}, month={month}");

	let stmt = q.prep(sql).await?;
	let params = params! { stock_code, year };
	let ent: Option<EntityRow> = q.exec_first(&stmt, params).await?;
	let fi = ent.map(FinancialInfo::try_from).transpose()?;
	Ok(fi)
}

async fn list(q: &mut QueryObject<'_>, stock_code: &str) -> Result<Vec<FinancialInfo>, Error> {
	let sql = format!("SELECT {FIELDS} FROM {TABLE} WHERE stock_code=:stock_code ORDER BY year");
	log::debug!("{sql} -- {{stock_code={stock_code}}}");

	let stmt = q.prep(sql).await?;
	let params = params! { stock_code };
	let rows: Vec<EntityRow> = q.exec(&stmt, params).await?;
	rows.into_repo_result()
}

async fn insert(q: &mut QueryObject<'_>, quarter: &FinancialInfo) -> Result<(), Error> {
	let values = SqlValues::from(quarter);
	let insert_clause = values.with_named_binding_holder();
	let sql = format!("INSERT INTO {TABLE} SET {insert_clause}");
	log::debug!("{} -- {}", sql, values);

	let stmt = q.prep(sql).await?;
	q.exec_drop(stmt, values.params()).await?;
	Ok(())
}

async fn update(q: &mut QueryObject<'_>, quarter: &mut FinancialInfo, data: FiQuarterData) -> Result<(), Error> {
	let values = SqlValues::from(&data);
	let insert_clause = values.with_named_binding_holder();
	let sql = format!("UPDATE {TABLE} SET {insert_clause} WHERE stock_code=:stock_code AND year=:year AND month=:month");
	log::debug!("{} -- {}, stock_code={}, year={}, month={}", sql, values, quarter.stock_code, quarter.year_month.year, quarter.year_month.month);

	let mut params = values.params();
	params.push(("stock_code".into(), quarter.stock_code.to_owned().into()));
	params.push(("year".into(), quarter.year_month.year.into()));
	params.push(("month".into(), quarter.year_month.month.into()));

	let stmt = q.prep(sql).await?;
	q.exec_drop(stmt, params).await?;

	quarter.sales = data.sales;
	quarter.profit = data.profit;
	quarter.net_income = data.net_income;
	quarter.dividend = data.dividend;
	quarter.dividend_yield = data.dividend_yield;
	Ok(())
}

impl<'a> From<&'a FiQuarterData> for SqlValues<'a> {
	fn from(data: &'a FiQuarterData) -> Self {
		SqlValues::default()
			.with("sales", data.sales)
			.with("profit", data.profit)
			.with("net_income", data.net_income)
			.with("dividend", data.dividend)
			.with("dividend_yield", data.dividend_yield)
	}
}
