use async_trait::async_trait;

use crate::types::{Error, YearMonth};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct FinancialInfo {
	/// 단축코드
	pub stock_code: String,
	/// 년월
	pub year_month: YearMonth,
	/// 매출액
	pub sales: Option<f32>,
	/// 영업이익
	pub profit: Option<f32>,
	/// 당기순이익
	pub net_income: Option<f32>,
	/// 주당배당금
	pub dividend: Option<f32>,
	/// 배당수익률
	pub dividend_yield: Option<f32>,
}

pub struct FiAnnualData {
	/// 기준년월
	pub month: u8,
	/// 매출액
	pub sales: Option<f32>,
	/// 영업이익
	pub profit: Option<f32>,
	/// 당기순이익
	pub net_income: Option<f32>,
	/// 주당배당금
	pub dividend: Option<f32>,
	/// 배당수익률
	pub dividend_yield: Option<f32>,
}
impl From<&FinancialInfo> for FiAnnualData {
	fn from(info: &FinancialInfo) -> Self {
		Self {
			month: info.year_month.month,
			sales: info.sales,
			profit: info.profit,
			net_income: info.net_income,
			dividend: info.dividend,
			dividend_yield: info.dividend_yield,
		}
	}
}

pub struct FiQuarterData {
	/// 매출액
	pub sales: Option<f32>,
	/// 영업이익
	pub profit: Option<f32>,
	/// 당기순이익
	pub net_income: Option<f32>,
	/// 주당배당금
	pub dividend: Option<f32>,
	/// 배당수익률
	pub dividend_yield: Option<f32>,
}
impl From<&FinancialInfo> for FiQuarterData {
	fn from(info: &FinancialInfo) -> Self {
		Self {
			sales: info.sales,
			profit: info.profit,
			net_income: info.net_income,
			dividend: info.dividend,
			dividend_yield: info.dividend_yield,
		}
	}
}

#[async_trait]
pub trait FiAnnualsDao {
	async fn find(&self, stock_code: &str, year: u16) -> Result<Option<FinancialInfo>, Error>;
	async fn list(&self, stock_code: &str) -> Result<Vec<FinancialInfo>, Error>;
	async fn insert(&self, annual: &FinancialInfo) -> Result<(), Error>;
	async fn update(&self, annual: &mut FinancialInfo, data: FiAnnualData) -> Result<(), Error>;
}

#[async_trait]
pub trait FiQuartersDao {
	async fn find(&self, stock_code: &str, year: u16, month: u8) -> Result<Option<FinancialInfo>, Error>;
	async fn list(&self, stock_code: &str) -> Result<Vec<FinancialInfo>, Error>;
	async fn insert(&self, quarter: &FinancialInfo) -> Result<(), Error>;
	async fn update(&self, quarter: &mut FinancialInfo, data: FiQuarterData) -> Result<(), Error>;
}
