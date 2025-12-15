use std::ops::Range;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::types::Error;

pub struct StockPrice {
	/// 단축코드
	pub stock_code: String,
	/// 일자
	pub ord_date: NaiveDate,
	/// 시가
	pub opening: Option<u32>,
	/// 고가
	pub highest: Option<u32>,
	/// 저가
	pub lowest: Option<u32>,
	/// 종가
	pub closing: Option<u32>,
	/// 전일대비
	pub diff: Option<i32>,
}

pub struct StockPriceRange {
	/// 고가
	pub highest: Option<u32>,
	/// 저가
	pub lowest: Option<u32>,
}

#[async_trait]
pub trait StockPricesDao {
	async fn latest(&self, code: &str) -> Result<Option<StockPrice>, Error>;
	async fn range(&self, code: &str, range: Range<NaiveDate>) -> Result<Option<StockPriceRange>, Error>;
}
