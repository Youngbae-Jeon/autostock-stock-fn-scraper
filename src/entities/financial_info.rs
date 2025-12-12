use async_trait::async_trait;

use crate::types::Error;

pub struct FiAnnual {
	/// 단축코드
	pub stock_code: String,
	/// 년도
	pub year: u16,
	/// 매출액
	pub sales: f64,
	/// 영업이익
	pub profit: f64,
	/// 당기순이익
	pub net_income: f64,
	/// 주당배당금
	pub dividend: f64,
	/// 배당수익률
	pub dividend_yield: f64,
}

pub struct FiQuarter {
	/// 단축코드
	pub stock_code: String,
	/// 년도
	pub year: u16,
	/// 분기
	pub quarter: u8,
	/// 매출액
	pub sales: f64,
	/// 영업이익
	pub profit: f64,
	/// 당기순이익
	pub net_income: f64,
	/// 주당배당금
	pub dividend: f64,
	/// 배당수익률
	pub dividend_yield: f64,
}

pub struct FiData {
	/// 매출액
	pub sales: f64,
	/// 영업이익
	pub profit: f64,
	/// 당기순이익
	pub net_income: f64,
	/// 주당배당금
	pub dividend: f64,
	/// 배당수익률
	pub dividend_yield: f64,
}

#[async_trait]
pub trait FiAnnualsDao {
	async fn find(&self, stock_code: &str, year: u16) -> Result<Option<FiAnnual>, Error>;
	async fn list(&self) -> Result<Vec<FiAnnual>, Error>;
	async fn insert(&self, annual: FiAnnual) -> Result<(), Error>;
	async fn update(&self, annual: &mut FiAnnual, data: FiData) -> Result<(), Error>;
}

#[async_trait]
pub trait FiQuartersDao {
	async fn find(&self, stock_code: &str, year: u16, quarter: u8) -> Result<Option<FiQuarter>, Error>;
	async fn list(&self) -> Result<Vec<FiQuarter>, Error>;
	async fn insert(&self, quarter: FiQuarter) -> Result<(), Error>;
	async fn update(&self, quarter: &mut FiQuarter, data: FiData) -> Result<(), Error>;
}
