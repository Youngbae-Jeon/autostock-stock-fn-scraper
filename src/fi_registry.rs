use crate::{entities::{EntityDao, FinancialInfo}, repository::Repo, types::{Error, YearMonth}};

pub struct Financials {
	pub annuals: FinancialInfoRegistry,
	pub quarters: FinancialInfoRegistry,
}
impl Financials {
	pub fn new(stock_code: &str) -> Self {
		Self {
			annuals: FinancialInfoRegistry::new(stock_code),
			quarters: FinancialInfoRegistry::new(stock_code),
		}
	}

	pub async fn save(&self, repo: &Repo) -> Result<(), Error> {
		log::debug!("annuals: {}, quarters: {}", self.annuals.len(), self.quarters.len());
		if !self.annuals.is_empty() {
			log::debug!("save_annuals");
			self.save_annuals(repo).await?;
		}
		if !self.quarters.is_empty() {
			log::debug!("save_quarters");
			self.save_quarters(repo).await?;
		}
		Ok(())
	}

	async fn save_annuals(&self, repo: &Repo) -> Result<(), Error> {
		let mut list = repo.fi_annuals().list(&self.annuals.stock_code).await?;
		for annual in self.annuals.iter() {
			match list.iter_mut().find(|fi| fi.year_month.year == annual.year_month.year) {
				Some(old) => {
					if old != annual {
						repo.fi_annuals().update(old, annual.into()).await?;
					}
				}
				None => {
					repo.fi_annuals().insert(annual).await?;
				}
			}
		}
		Ok(())
	}

	async fn save_quarters(&self, repo: &Repo) -> Result<(), Error> {
		let mut list = repo.fi_quarters().list(&self.quarters.stock_code).await?;
		for quarter in self.quarters.iter() {
			match list.iter_mut().find(|fi| fi.year_month == quarter.year_month) {
				Some(old) => {
					if old != quarter {
						repo.fi_quarters().update(old, quarter.into()).await?;
					}
				}
				None => {
					repo.fi_quarters().insert(quarter).await?;
				}
			}
		}
		Ok(())
	}
}

pub struct FinancialInfoRegistry {
	stock_code: String,
	pub list: Vec<FinancialInfo>,
}
impl FinancialInfoRegistry {
	pub fn new(stock_code: &str) -> Self {
		Self {
			stock_code: stock_code.to_string(),
			list: Vec::new(),
		}
	}

	pub fn register(&mut self, year_month: YearMonth, data_name: &str, value: Option<f32>) {
		let fi = self.iter_mut().find(|fi| fi.year_month == year_month);
		if let Some(fi) = fi {
			FinancialInfoRegistry::set_fi_property(fi, data_name, value);
		} else {
			let mut fi = FinancialInfo {
				stock_code: self.stock_code.clone(),
				year_month,
				..Default::default()
			};
			FinancialInfoRegistry::set_fi_property(&mut fi, data_name, value);
			self.list.push(fi);
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &FinancialInfo> {
		self.list.iter()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut FinancialInfo> {
		self.list.iter_mut()
	}

	pub fn set_fi_property(fi: &mut FinancialInfo, data_name: &str, value: Option<f32>) {
		match data_name {
			"매출액" => fi.sales = value,
			"영업이익" => fi.profit = value,
			"당기순이익" => fi.net_income = value,
			"주당배당금(원)" => fi.dividend = value,
			"시가배당률(%)" => fi.dividend_yield = value,
			_ => (),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}
}
