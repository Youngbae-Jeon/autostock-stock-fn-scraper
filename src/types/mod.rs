use std::{fmt::Display, str::FromStr};

use chrono::{Datelike, NaiveDate};

mod errors;

pub use errors::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Market {
	KOSPI,
	KOSDAQ,
	ETF,
}

impl Market {
	pub fn as_str(&self) -> &'static str {
		match self {
			Market::KOSPI => "KOSPI",
			Market::KOSDAQ => "KOSDAQ",
			Market::ETF => "ETF",
		}
	}
}

impl FromStr for Market {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"KOSPI" => Ok(Market::KOSPI),
			"KOSDAQ" => Ok(Market::KOSDAQ),
			"ETF" => Ok(Market::ETF),
			_ => Err(format!("Unknown Market Representation `{s}`").into()),
		}
	}
}

impl TryFrom<&str> for Market {
	type Error = Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		Market::from_str(value)
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct YearMonth {
	pub year: u16,
	pub month: u8,
}
impl YearMonth {
	pub fn new(year: u16, month: u8) -> Self {
		Self { year, month }
	}
}
impl From<NaiveDate> for YearMonth {
	fn from(date: NaiveDate) -> Self {
		Self {
			year: date.year() as u16,
			month: date.month() as u8,
		}
	}
}
impl FromStr for YearMonth {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<&str> = s.split(&['.', '-']).collect();
		if parts.len() != 2 {
			return Err(format!("Invalid YearMonth format `{s}`").into());
		}
		let year = parts[0].parse::<u16>().map_err(|_| format!("Invalid YearMonth format `{s}`"))?;
		let month = parts[1].parse::<u8>().map_err(|_| format!("Invalid YearMonth format `{s}`"))?;
		if month < 1 || month > 12 {
			return Err(format!("Invalid YearMonth format `{s}`").into());
		}
		Ok(Self { year, month })
	}
}
impl Display for YearMonth {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{:02}", self.year, self.month)
	}
}
impl PartialOrd for YearMonth {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
impl Ord for YearMonth {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.year.cmp(&other.year).then_with(|| self.month.cmp(&other.month))
	}
}
