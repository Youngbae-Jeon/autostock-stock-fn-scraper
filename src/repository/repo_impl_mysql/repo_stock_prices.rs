use std::ops::Range;

use async_trait::async_trait;
use chrono::NaiveDate;
use mysql_async::{params, prelude::FromRow};
use repo_helper::{database_table, mysql::QueryObject};

use crate::{entities::{StockPrice, StockPriceRange, StockPricesDao}, types::Error};
use crate::repository::repo_impl_mysql::{repo::RepoImpl, repo_tx::RepoTxImpl};


#[async_trait]
impl StockPricesDao for RepoImpl {
	async fn latest(&self, code: &str) -> Result<Option<StockPrice>, Error> {
		let mut q = self.get_query_object().await?;
		latest(&mut q, code).await
	}
	async fn oldest_and_latest(&self, code: &str) -> Result<Option<(StockPrice, StockPrice)>, Error> {
		let mut q = self.get_query_object().await?;
		oldest_and_latest(&mut q, code).await
	}
	async fn range(&self, code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error> {
		let mut q = self.get_query_object().await?;
		query_range(&mut q, code, range).await
	}
	async fn delete_before(&self, code: &str, date: NaiveDate) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		delete_before(&mut q, code, date).await
	}
}

#[async_trait]
impl StockPricesDao for RepoTxImpl {
	async fn latest(&self, code: &str) -> Result<Option<StockPrice>, Error> {
		let mut q = self.get_query_object().await?;
		latest(&mut q, code).await
	}
	async fn oldest_and_latest(&self, code: &str) -> Result<Option<(StockPrice, StockPrice)>, Error> {
		let mut q = self.get_query_object().await?;
		oldest_and_latest(&mut q, code).await
	}
	async fn range(&self, code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error> {
		let mut q = self.get_query_object().await?;
		query_range(&mut q, code, range).await
	}
	async fn delete_before(&self, code: &str, date: NaiveDate) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		delete_before(&mut q, code, date).await
	}
}


database_table! {
	#[table_name = "item_price", derive(FromRow)]
	EntityRow {
		code: String,
		ord_date: NaiveDate,
		opening: Option<u32>,
		highest: Option<u32>,
		lowest: Option<u32>,
		closing: Option<u32>,
		diff: Option<i32>,
	}
}
impl TryFrom<EntityRow> for StockPrice {
	type Error = Error;

	fn try_from(value: EntityRow) -> Result<Self, Self::Error> {
		Ok(Self {
			stock_code: value.code,
			ord_date: value.ord_date,
			opening: value.opening,
			highest: value.highest,
			lowest: value.lowest,
			closing: value.closing,
			diff: value.diff,
		})
	}
}


const TABLE: &str = EntityRow::TABLE_NAME;
const FIELDS: &str = EntityRow::TABLE_FIELDS;


async fn latest(q: &mut QueryObject<'_>, stock_code: &str) -> Result<Option<StockPrice>, Error> {
	let sql = format!("SELECT {FIELDS} FROM {TABLE} WHERE code=:stock_code ORDER BY ord_date desc LIMIT 1");
	log::debug!("{sql} -- stock_code={stock_code}");

	let stmt = q.prep(sql).await?;
	let params = params! { stock_code };
	let ent: Option<EntityRow> = q.exec_first(&stmt, params).await?;
	let fi = ent.map(StockPrice::try_from).transpose()?;
	Ok(fi)
}

async fn oldest_and_latest(q: &mut QueryObject<'_>, stock_code: &str) -> Result<Option<(StockPrice, StockPrice)>, Error> {
	let sql = format!("SELECT {FIELDS} FROM (\
		SELECT a.* FROM {TABLE} AS a \
		JOIN (\
		SELECT MIN(ord_date) AS min_ord_date, MAX(ord_date) AS max_ord_date FROM {TABLE} WHERE code=?\
		) AS b ON a.ord_date=b.min_ord_date OR a.ord_date=b.max_ord_date \
		WHERE a.code=?\
		) c ORDER BY ord_date");
	log::debug!("{sql} -- [{stock_code}, {stock_code}]");

	let stmt = q.prep(sql).await?;
	let params = vec! [ stock_code, stock_code ];
	let mut rows: Vec<EntityRow> = q.exec(&stmt, params).await?;
	let len = rows.len();
	match len {
		0 => Ok(None),
		1 => {
			let ent1: StockPrice = rows.remove(0).try_into()?;
			let ent0 = ent1.clone();
			Ok(Some((ent0, ent1)))
		}
		2 => {
			let ent1: StockPrice = rows.remove(1).try_into()?;
			let ent0: StockPrice = rows.remove(0).try_into()?;
			Ok(Some((ent0, ent1)))
		}
		_ => Err(Error::from("unexpected number of rows")),
	}
}

#[derive(FromRow)]
struct StockPriceRangeEntityRow {
	highest: Option<u32>,
	lowest: Option<u32>,
}
impl TryFrom<StockPriceRangeEntityRow> for StockPriceRange {
	type Error = Error;

	fn try_from(value: StockPriceRangeEntityRow) -> Result<Self, Self::Error> {
		Ok(Self {
			highest: value.highest,
			lowest: value.lowest,
		})
	}
}

async fn query_range(q: &mut QueryObject<'_>, stock_code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error> {
	let ord_date_start = range.start;
	let ord_date_end = range.end;

	let sql = format!("SELECT MAX(highest) AS highest, MIN(lowest) AS lowest FROM {TABLE} WHERE code=:stock_code AND ord_date>=:ord_date_start AND ord_date<:ord_date_end AND highest>0 AND lowest>0");
	log::debug!("{sql} -- code={stock_code}, ord_date_start={ord_date_start}, ord_date_end={ord_date_end}");

	let stmt = q.prep(sql).await?;
	let params = params! { stock_code, ord_date_start, ord_date_end };
	let ent: Option<StockPriceRangeEntityRow> = q.exec_first(&stmt, params).await?;
	let price_range = ent.map(StockPriceRange::try_from).transpose()?;
	Ok(price_range)
}

async fn delete_before(q: &mut QueryObject<'_>, stock_code: &str, date: NaiveDate) -> Result<(), Error> {
	let sql = format!("DELETE FROM {TABLE} WHERE code=:stock_code AND ord_date<:date");
	log::debug!("{sql} -- stock_code={stock_code}, date={date}");

	let stmt = q.prep(sql).await?;
	let params = params! { stock_code, date };
	q.exec_drop(&stmt, params).await?;
	Ok(())
}
