use async_trait::async_trait;
use chrono::NaiveDate;

use crate::types::{Error, Market};

pub struct ProductItem {
	/// 단축코드
	pub code: String,
	/// 기준일자
	pub info_date: NaiveDate,
	/// 종목명
	pub name: String,
	/// 시장구분
	pub market: Market,
	/// 종목코드
	pub std_code: Option<String>,
	/// 상장일자
	pub list_date: Option<NaiveDate>,
	/// 주식종류
	pub kind: Option<String>,
	/// 증권구분
	pub secu_group: Option<String>,
	/// 소속부
	pub sect: Option<String>,
	/// 액면가
	pub par: Option<u32>,
	/// 상장주식수
	pub list_shares: Option<u64>,
	/// 기초지수명
	pub etf_obj_idx: Option<String>,
	/// 기초지수산출기관
	pub etf_idx_inst: Option<String>,
	/// 추적배수
	pub etf_idx_multiplier: Option<String>,
	/// 복제방법
	pub etf_replica_method: Option<String>,
	/// 기초시장분류
	pub etf_idx_market: Option<String>,
	/// 기초자산분류
	pub etf_idx_asset: Option<String>,
	/// 운용사
	pub etf_op_company: Option<String>,
	/// 총보수
	pub etf_fee_rate: Option<f32>,
	/// 과세유형
	pub etf_tax_type: Option<String>,
}

#[async_trait]
pub trait ProductItemsDao {
	async fn list(&self) -> Result<Vec<ProductItem>, Error>;
}

pub struct Stock {
	/// 단축코드
	pub code: String,
	/// 기준일자
	pub info_date: NaiveDate,
	/// 종목명
	pub name: String,
	/// 시장구분
	pub market: Market,
	/// 종목코드
	pub std_code: Option<String>,
	/// 상장일자
	pub list_date: Option<NaiveDate>,
	/// 주식종류
	pub kind: Option<String>,
	/// 증권구분
	pub secu_group: Option<String>,
	/// 소속부
	pub sect: Option<String>,
	/// 액면가
	pub par: Option<u32>,
	/// 상장주식수
	pub list_shares: Option<u64>,
}

#[async_trait]
pub trait StocksDao {
	async fn list(&self) -> Result<Vec<Stock>, Error>;
}
