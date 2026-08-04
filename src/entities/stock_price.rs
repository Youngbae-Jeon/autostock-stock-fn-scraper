use std::{ops::Range};

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::types::Error;

#[derive(Clone)]
pub struct StockPrice {
	/// 일자
	pub ord_date: NaiveDate,
	/// 시가
	pub opening: u32,
	/// 고가
	pub highest: u32,
	/// 저가
	pub lowest: u32,
	/// 종가
	pub closing: u32,
	/// 전일대비
	pub diff: i32,
}

pub struct StockPriceRange {
	/// 고가
	pub highest: u32,
	/// 저가
	pub lowest: u32,
}

#[async_trait]
pub trait StockPricesDao {
	async fn latest(&self, code: &str) -> Result<Option<StockPrice>, Error>;
	async fn oldest_and_latest(&self, code: &str) -> Result<Option<(StockPrice, StockPrice)>, Error>;
	async fn range(&self, code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error>;
	async fn delete_all(&self, code: &str) -> Result<(), Error>;
	async fn insert_all(&self, code: &str, prices: &[StockPrice]) -> Result<(), Error>;
}
