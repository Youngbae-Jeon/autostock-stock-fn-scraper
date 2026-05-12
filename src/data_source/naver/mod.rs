mod stock_financials;
mod stock_prices;

use std::time::Duration;

use ratelimit::{Ratelimiter, TryWaitError};
use reqwest::IntoUrl;

use crate::types::Error;

pub use stock_financials::query_stock_financials;
pub use stock_prices::query_stock_prices;

lazy_static::lazy_static! {
	static ref NAVER_RATELIMITER: Ratelimiter = Ratelimiter::builder(1)
		.period(std::time::Duration::from_millis(500))
		.initial_available(1)
		.build()
		.unwrap();
}

async fn request<U: IntoUrl>(url: U) -> Result<String, Error> {
	while let Err(TryWaitError::Insufficient(dur)) = NAVER_RATELIMITER.try_wait() {
		tokio::time::sleep(dur).await;
	}

	let client = reqwest::Client::new();
	let resp = client.get(url)
		.header("Content-Type", "application/x-www-form-urlencoded")
		.timeout(Duration::from_secs(5))
		.send()
		.await
		.map_err(|e| e.to_string())?;

	let text = resp.text()
		.await
		.map_err(|e| e.to_string())?;
	Ok(text)
}
