use std::str::FromStr;

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
