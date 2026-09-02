use std::ops::Range;

use async_trait::async_trait;
use chrono::NaiveDate;
use mysql_async::{params, prelude::{FromRow, Queryable}};
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
	async fn delete_all(&self, code: &str) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		delete_all(&mut q, code).await
	}
	async fn insert_all(&self, code: &str, prices: &[StockPrice]) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		insert_all(&mut q, code, prices).await
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
	async fn delete_all(&self, code: &str) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		delete_all(&mut q, code).await
	}
	async fn insert_all(&self, code: &str, prices: &[StockPrice]) -> Result<(), Error> {
		let mut q = self.get_query_object().await?;
		insert_all(&mut q, code, prices).await
	}
}


database_table! {
	#[table_name = "item_price", derive(FromRow)]
	EntityRow {
		ord_date: NaiveDate,
		opening: u32,
		highest: u32,
		lowest: u32,
		closing: u32,
		diff: i32,
	}
}
impl TryFrom<EntityRow> for StockPrice {
	type Error = Error;

	fn try_from(value: EntityRow) -> Result<Self, Self::Error> {
		Ok(Self {
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


async fn latest(q: &mut QueryObject<'_>, code: &str) -> Result<Option<StockPrice>, Error> {
	let sql = format!("SELECT {FIELDS} FROM {TABLE} WHERE code=:code ORDER BY ord_date desc LIMIT 1");
	log::debug!("{sql} -- code={code}");

	let stmt = q.prep(sql).await?;
	let params = params! { code };
	let ent: Option<EntityRow> = q.exec_first(&stmt, params).await?;
	let fi = ent.map(StockPrice::try_from).transpose()?;
	Ok(fi)
}

async fn oldest_and_latest(q: &mut QueryObject<'_>, code: &str) -> Result<Option<(StockPrice, StockPrice)>, Error> {
	let sql = format!("SELECT {FIELDS} FROM (\
		SELECT a.* FROM {TABLE} AS a \
		JOIN (\
		SELECT MIN(ord_date) AS min_ord_date, MAX(ord_date) AS max_ord_date FROM {TABLE} WHERE code=?\
		) AS b ON a.ord_date=b.min_ord_date OR a.ord_date=b.max_ord_date \
		WHERE a.code=?\
		) c ORDER BY ord_date");
	log::debug!("{sql} -- [{code}, {code}]");

	let stmt = q.prep(sql).await?;
	let params = vec! [ code, code ];
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
	highest: u32,
	lowest: u32,
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

async fn query_range(q: &mut QueryObject<'_>, code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error> {
	let ord_date_start = range.start;
	let ord_date_end = range.end;

	let sql = format!("SELECT MAX(highest) AS highest, MIN(lowest) AS lowest FROM {TABLE} WHERE code=:code AND ord_date>=:ord_date_start AND ord_date<:ord_date_end AND highest>0 AND lowest>0");
	log::debug!("{sql} -- code={code}, ord_date_start={ord_date_start}, ord_date_end={ord_date_end}");

	let stmt = q.prep(sql).await?;
	let params = params! { code, ord_date_start, ord_date_end };
	let ent: Option<StockPriceRangeEntityRow> = q.exec_first(&stmt, params).await?;
	let price_range = ent.map(StockPriceRange::try_from).transpose()?;
	Ok(price_range)
}

async fn delete_all(q: &mut QueryObject<'_>, code: &str) -> Result<(), Error> {
	let sql = format!("DELETE FROM {TABLE} WHERE code=:code");
	log::debug!("{sql} -- code={code}");

	let stmt = q.prep(sql).await?;
	let params = params! { code };
	q.exec_drop(&stmt, params).await?;
	Ok(())
}

async fn insert_all(q: &mut QueryObject<'_>, code: &str, prices: &[StockPrice]) -> Result<(), Error> {
	let sql = format!("INSERT INTO {TABLE} (code, ord_date, opening, highest, lowest, closing, diff) VALUES (:code, :ord_date, :opening, :highest, :lowest, :closing, :diff)");
	let prices_txt = prices.iter()
		.map(|p| {
			format!(
				"[{}, {}, {}, {}, {}, {}, {}]",
				code, p.ord_date, p.opening, p.highest, p.lowest, p.closing, p.diff
			)
		})
		.collect::<Vec<_>>().join(", ");
	log::debug!("{sql} -- code={code}, prices=[{prices_txt}]");

	let stmt = q.prep(sql).await?;
	q.exec_batch(stmt, prices.iter().map(|p| {
		params! {
			"code" => code,
			"ord_date" => p.ord_date,
			"opening" => p.opening,
			"highest" => p.highest,
			"lowest" => p.lowest,
			"closing" => p.closing,
			"diff" => p.diff,
		}
	})).await?;
	Ok(())
}
