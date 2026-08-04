use chrono::{Duration, NaiveDate};
use ratelimit::TryWaitError;

use crate::{entities::StockPrice, types::Error};

use super::NAVER_RATELIMITER;

// https://api.stock.naver.com/chart/domestic/item/005930/day?startDateTime=202410250000&endDateTime=202603250952
pub async fn query_stock_prices(stock_code: &str, start: NaiveDate, end: NaiveDate) -> Result<Vec<StockPrice>, Error> {
	while let Err(TryWaitError::Insufficient(dur)) = NAVER_RATELIMITER.try_wait() {
		tokio::time::sleep(dur).await;
	}

	let params = [
		("startDateTime", start.format("%Y%m%d0000").to_string()),
		("endDateTime", (end - Duration::days(1)).format("%Y%m%d0000").to_string()),
	];
	let url = reqwest::Url::parse_with_params(&format!("https://api.stock.naver.com/chart/domestic/item/{stock_code}/day"), params)
		.map_err(|e| e.to_string())?;
	log::debug!("Fetching stock prices from NAVER: {}", url);

	// let text = reqwest::get(url).await
	// 	.map_err(|e| e.to_string())?
	// 	.text().await
	// 	.map_err(|e| e.to_string())?;
	// log::debug!("text={text}");
	// let list: Vec<NaverStockChartPrice> = serde_json::from_str(&text)
	// 	.map_err(|e| e.to_string())?;

	let list: Vec<NaverStockChartPrice> = reqwest::get(url).await
		.map_err(|e| e.to_string())?
		.json().await
		.map_err(|e| e.to_string())?;
	log::debug!("{} daily prices fetched ({} ~ {})", list.len(), list.first().map(|p| p.local_date.clone()).unwrap_or_default(), list.last().map(|p| p.local_date.clone()).unwrap_or_default());

	let mut prices: Vec<StockPrice> = Vec::with_capacity(list.len());
	for item in list {
		prices.push(StockPrice::try_from(item)?);
	}
	Ok(prices)
}

// {"localDate":"20260508","closePrice":268500.0,"openPrice":260000.0,"highPrice":270000.0,"lowPrice":260000.0,"accumulatedTradingVolume":25875880,"foreignRetentionRate":49.21}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NaverStockChartPrice {
	local_date: String,
	close_price: f64,
	open_price: f64,
	high_price: f64,
	low_price: f64,
	// accumulated_trading_volume: u64,
	// foreign_retention_rate: f64,
}

impl TryFrom<NaverStockChartPrice> for StockPrice {
	type Error = Error;

	fn try_from(value: NaverStockChartPrice) -> Result<Self, Self::Error> {
		Ok(StockPrice {
			ord_date: NaiveDate::parse_from_str(&value.local_date, "%Y%m%d")?,
			opening: value.open_price as u32,
			highest: value.high_price as u32,
			lowest: value.low_price as u32,
			closing: value.close_price as u32,
			diff: 0,
		})
	}
}
