use std::{collections::LinkedList, fs::File, io, ops::Range};

use chrono::{Duration, Local, NaiveDate};
use csv::{QuoteStyle, WriterBuilder};
use serde::Serialize;
use stock_fn_scraper::{entities::{EntityDao, FinancialInfo, Stock, StockPrice, StockPriceRange}, logger, repository::{self, DatabaseConfig, Repo}, types::{Error, YearMonth}};



#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
	logger::prepare();

	let db_conf = DatabaseConfig::from_env();
	let repo = repository::create(&db_conf).await;

	let today = Local::now().date_naive();
	let stocks = repo.stocks().list().await.unwrap();

	let mut data_list = LinkedList::<Data>::new();
	for stock in stocks.into_iter() {
		if today - stock.info_date > chrono::Duration::days(10) {
			log::info!("Stock `{}|{}` Outdated and skipped (date:{})", stock.code, stock.name, stock.info_date);
			continue;
		}

		let stock_code = stock.code.clone();
		let stock_name = stock.name.clone();
		match fetch_data(stock, &repo).await {
			Ok(data) => data_list.push_back(data),
			Err(err) => log::error!("Failed to fetch data for stock `{}|{}': {}", stock_code, stock_name, err),
		}
	}

	create_csv(&data_list).await.unwrap();
}

struct Data {
	stock: Stock,
	price_latest: Option<StockPrice>,
	price_range: Option<StockPriceRange>,
	annuals: Vec<FinancialInfo>,
	quarters: Vec<FinancialInfo>,
}

async fn fetch_data(stock: Stock, repo: &Repo) -> Result<Data, Error> {
	let price_latest = repo.stock_prices().latest(&stock.code).await?;
	let recent_five_years = {
		let end_date = stock.info_date + Duration::days(1);
		let start_date = end_date - Duration::days(365 * 5);
		Range { start: start_date,  end: end_date }
	};
	let price_range = repo.stock_prices().range(&stock.code, recent_five_years).await?;
	let annuals: Vec<_> = repo.fi_annuals().list(&stock.code).await?
		.into_iter().rev().take(3).rev().collect();
	let quarters: Vec<_> = repo.fi_quarters().list(&stock.code).await?
		.into_iter().rev().take(3).rev().collect();
	Ok(Data { stock, price_latest, price_range, annuals, quarters })
}

#[derive(Default, Serialize)]
struct Record {
	/// 종목코드
	#[serde(rename = "종목코드")]
	code: String,
	/// 종목명
	#[serde(rename = "종목명")]
	name: String,
	/// 시장구분
	#[serde(rename = "시장구분")]
	market: &'static str,
	/// 기준일자
	#[serde(rename = "기준일자")]
	date: NaiveDate,
	/// 종가
	#[serde(rename = "종가")]
	price: Option<u32>,
	/// 시가총액
	#[serde(rename = "시가총액(억)")]
	market_cap: Option<u32>,
	/// 최근5년 최고가
	#[serde(rename = "최근5년 최고가")]
	highest_in_recent: Option<u32>,
	/// 최근5년 최저가
	#[serde(rename = "최근5년 최저가")]
	lowest_in_recent: Option<u32>,
	/// 연간실적 기준년월 1
	#[serde(rename = "연간실적(Y-3)")]
	y1_date: Option<YearMonth>,
	/// 매출액 - 연간실적 기준년월 1
	#[serde(rename = "매출액(억)")]
	y1_sales: Option<i32>,
	/// 영업이익 - 연간실적 기준년월 1
	#[serde(rename = "영업이익(억)")]
	y1_profit: Option<i32>,
	#[serde(rename = "연간실적(Y-2)")]
	y2_date: Option<YearMonth>,
	#[serde(rename = "매출액(억)")]
	y2_sales: Option<i32>,
	#[serde(rename = "영업이익(억)")]
	y2_profit: Option<i32>,
	#[serde(rename = "연간실적(Y-1)")]
	y3_date: Option<YearMonth>,
	#[serde(rename = "매출액(억)")]
	y3_sales: Option<i32>,
	#[serde(rename = "영업이익(억)")]
	y3_profit: Option<i32>,
	/// 분기실적 기준년월 1
	#[serde(rename = "분기실적(Q-3)")]
	q1_date: Option<YearMonth>,
	/// 매출액 - 분기실적 기준년월 1
	#[serde(rename = "매출액(억)")]
	q1_sales: Option<i32>,
	/// 영업이익 - 분기실적 기준년월 1
	#[serde(rename = "영업이익(억)")]
	q1_profit: Option<i32>,
	#[serde(rename = "분기실적(Q-2)")]
	q2_date: Option<YearMonth>,
	#[serde(rename = "매출액(억)")]
	q2_sales: Option<i32>,
	#[serde(rename = "영업이익(억)")]
	q2_profit: Option<i32>,
	#[serde(rename = "분기실적(Q-1)")]
	q3_date: Option<YearMonth>,
	#[serde(rename = "매출액(억)")]
	q3_sales: Option<i32>,
	#[serde(rename = "영업이익(억)")]
	q3_profit: Option<i32>,
}

async fn create_csv(data_list: &LinkedList<Data>) -> Result<(), Error> {
	let file = File::create("종목데이터.csv")?;
	// let mut writer = Writer::from_writer(file);
	let mut writer = WriterBuilder::new()
		.quote_style(QuoteStyle::NonNumeric) // Set the quoting style
		.from_writer(file);
		// .from_writer(io::stdout());
	log::info!("Writing {} data in CSV...", data_list.len());

	for data in data_list.iter() {
		let mut rec = Record::default();
		rec.code = data.stock.code.clone();
		rec.name = data.stock.name.clone();
		rec.market = data.stock.market.as_str();
		rec.date = data.stock.info_date;
		rec.price = data.price_latest.as_ref().and_then(|p| p.closing);
		rec.market_cap = rec.price.zip(data.stock.list_shares).map(|(price, shares)| (price as f64 * shares as f64 / 100000000 as f64).round() as u32);
		rec.highest_in_recent = data.price_range.as_ref().and_then(|r| r.highest);
		rec.lowest_in_recent = data.price_range.as_ref().and_then(|r| r.lowest);

		let i = data.annuals.len() as i32 - 1;
		if i >= 0 {
			rec.y3_date = Some(data.annuals[i as usize].year_month);
			rec.y3_sales = data.annuals[i as usize].sales.map(|v| v as i32);
			rec.y3_profit = data.annuals[i as usize].profit.map(|v| v as i32);
		}

		let i = data.annuals.len() as i32 - 2;
		if i >= 0 {
			rec.y2_date = Some(data.annuals[i as usize].year_month);
			rec.y2_sales = data.annuals[i as usize].sales.map(|v| v as i32);
			rec.y2_profit = data.annuals[i as usize].profit.map(|v| v as i32);
		}

		let i = data.annuals.len() as i32 - 3;
		if i >= 0 {
			rec.y1_date = Some(data.annuals[i as usize].year_month);
			rec.y1_sales = data.annuals[i as usize].sales.map(|v| v as i32);
			rec.y1_profit = data.annuals[i as usize].profit.map(|v| v as i32);
		}

		let i = data.quarters.len() as i32 - 1;
		if i >= 0 {
			rec.q3_date = Some(data.quarters[i as usize].year_month);
			rec.q3_sales = data.quarters[i as usize].sales.map(|v| v as i32);
			rec.q3_profit = data.quarters[i as usize].profit.map(|v| v as i32);
		}

		let i = data.quarters.len() as i32 - 2;
		if i >= 0 {
			rec.q2_date = Some(data.quarters[i as usize].year_month);
			rec.q2_sales = data.quarters[i as usize].sales.map(|v| v as i32);
			rec.q2_profit = data.quarters[i as usize].profit.map(|v| v as i32);
		}

		let i = data.quarters.len() as i32 - 3;
		if i >= 0 {
			rec.q1_date = Some(data.quarters[i as usize].year_month);
			rec.q1_sales = data.quarters[i as usize].sales.map(|v| v as i32);
			rec.q1_profit = data.quarters[i as usize].profit.map(|v| v as i32);
		}

		writer.serialize(rec)?;
	}

	writer.flush()?;
	Ok(())
}
