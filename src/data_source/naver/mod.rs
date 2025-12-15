use std::{collections::HashMap, ops::Range, time::Duration};

use ratelimit::Ratelimiter;
use reqwest::IntoUrl;
use scraper::{Element, ElementRef, Html, Selector};

use crate::{entities::FinancialInfo, fi_registry::Financials, types::{Error, YearMonth}};

lazy_static::lazy_static! {
	static ref NAVER_RATELIMITER: Ratelimiter = Ratelimiter::builder(1, std::time::Duration::from_millis(500))
		.initial_available(1)
		.build()
		.unwrap();
}

async fn request<U: IntoUrl>(url: U) -> Result<String, Error> {
	while let Err(dur) = NAVER_RATELIMITER.try_wait() {
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


pub async fn query_stock_financials(stock_code: &str) -> Result<Financials, Error> {
	let params = [("code", stock_code)];
	let url = reqwest::Url::parse_with_params("https://finance.naver.com/item/main.naver", params)
		.map_err(|e| e.to_string())?;
	let text = request(url).await?;
	let mut financials = parse_html_resp(&text, stock_code)?;
	financials.remove_duplicate();
	Ok(financials)
}

lazy_static::lazy_static! {
	static ref CAPTION_SELECTOR: Selector = Selector::parse("caption").unwrap();
	static ref THEAD_TR_SELECTOR: Selector = Selector::parse("thead tr").unwrap();
	static ref TBODY_TR_SELECTOR: Selector = Selector::parse("tbody tr").unwrap();
	static ref TH_SELECTOR: Selector = Selector::parse("th").unwrap();
	static ref TD_SELECTOR: Selector = Selector::parse("th,td").unwrap();
}

fn parse_html_resp(html: &str, stock_code: &str) -> Result<Financials, Error> {
	let document = Html::parse_document(html);
	let table = document.select(&CAPTION_SELECTOR)
		.filter(|el| el.text().next().unwrap_or("").contains("기업실적분석"))
		.filter_map(|el| el.parent_element().filter(|p| p.value().name() == "table"))
		.next();
	let Some(table) = table else {
		return Err("기업실적분석 테이블을 찾을 수 없습니다".into());
	};

	let mut thead_trs = table.select(&THEAD_TR_SELECTOR);
	let Some(tr) = thead_trs.next() else {
		return Err("Not found header row(0)".into());
	};

	let mut rowspans = HashMap::<usize, usize>::new();

	let mut annual_col_indices: Option<Range<usize>> = None;
	let mut quater_col_indices: Option<Range<usize>> = None;
	let mut ths = tr.select(&TH_SELECTOR);
	let mut col_idx = 0;
	loop {
		let Some(th) = ths.next() else {
			break
		};

		let rowspan = th.attr("rowspan")
			.map(|rowspan| rowspan.parse::<usize>().unwrap_or(1))
			.unwrap_or(1);
		if rowspan > 1 {
			rowspans.insert(col_idx, rowspan);
		}

		let colspan = th.attr("colspan")
			.map(|colspan| colspan.parse::<usize>().unwrap_or(1))
			.unwrap_or(1);

		let Some(text) = th.text().next() else {
			col_idx += colspan;
			continue
		};

		if text.contains("최근 연간 실적") {
			annual_col_indices = Some(col_idx..col_idx + colspan);
		} else if text.contains("최근 분기 실적") {
			quater_col_indices = Some(col_idx..col_idx + colspan);
		}
		col_idx += colspan;
	};

	let Some(annual_col_indices) = annual_col_indices else {
		return Err("Not found annual columns".into());
	};
	let Some(quater_col_indices) = quater_col_indices else {
		return Err("Not found quater columns".into());
	};

	// println!("Annual columns: {:?}", annual_col_indices);
	// println!("Quater columns: {:?}", quater_col_indices);

	let Some(tr) = thead_trs.next() else {
		return Err("Not found header row(1)".into());
	};

	let mut annual_columns = Vec::<(YearMonth, usize)>::new();
	let mut quarter_columns = Vec::<(YearMonth, usize)>::new();

	let mut ths = tr.select(&TH_SELECTOR);
	let mut col_idx = 0;
	loop {
		if let Some(rowspan) = rowspans.get(&col_idx) {
			let new_rowspan = rowspan -1;
			if new_rowspan > 1 {
				rowspans.insert(col_idx, new_rowspan);
			} else {
				rowspans.remove(&col_idx);
			}
			col_idx += 1;
			continue
		}

		let Some(th) = ths.next() else {
			break
		};

		let colspan = th.attr("colspan")
			.map(|colspan| colspan.parse::<usize>().unwrap_or(1))
			.unwrap_or(1);

		let text = get_text(th);
		if text.is_empty() || text.ends_with("(E)") {
			col_idx += colspan;
			continue;
		}

		if annual_col_indices.contains(&col_idx) {
			let year_month: YearMonth = text.parse()?;
			annual_columns.push((year_month, col_idx));
		}
		if quater_col_indices.contains(&col_idx) {
			let year_month: YearMonth = text.parse()?;
			quarter_columns.push((year_month, col_idx));
		}
		col_idx += colspan;
	}

	// println!("Annual columns: {:?}", annual_columns);
	// println!("Quarter columns: {:?}", quarter_columns);
	let mut financials = Financials::new(stock_code);

	let mut tbody_trs = table.select(&TBODY_TR_SELECTOR);
	loop {
		let Some(tr) = tbody_trs.next() else {
			break
		};
		// println!("TBODY TR: {:?}", tr);

		let mut tds = tr.select(&TD_SELECTOR);
		let mut col_idx = 0;
		let mut data_name: Option<String> = None;
		loop {
			if col_idx > 0 && data_name.is_none() {
				break
			}
			let Some(td) = tds.next() else {
				break
			};

			let colspan = td.attr("colspan")
				.map(|colspan| colspan.parse::<usize>().unwrap_or(1))
				.unwrap_or(1);

			let Some(text) = get_text_opt(td) else {
				col_idx += colspan;
				continue;
			};

			if col_idx == 0 {
				if text == "매출액" || text == "영업이익" || text == "당기순이익" || text == "주당배당금(원)" || text == "시가배당률(%)" {
					data_name = Some(text);
				}
				col_idx += colspan;
				continue
			}

			let annual_column = annual_columns.iter().find(|(_, ci)| *ci == col_idx);
			if let Some((year_month, _)) = annual_column {
				let value = text.replace(",", "").parse::<f32>().ok();
				// println!("Annual {year_month} {data_name:?} {text} ({value:?})");
				if let Some(data_name) = data_name.as_ref() && value.is_some() {
					financials.annuals.register(*year_month, data_name, value);
				};
			}

			let quarter_column = quarter_columns.iter().find(|(_, ci)| *ci == col_idx);
			if let Some((year_month, _)) = quarter_column {
				let value = text.replace(",", "").parse::<f32>().ok();
				// println!("Quarter {year_month} {data_name:?} {text} ({value:?})");
				if let Some(data_name) = data_name.as_ref() && value.is_some() {
					financials.quarters.register(*year_month, data_name, value);
				};
			}

			col_idx += colspan;
		}
	}

	Ok(financials)
}

fn get_text(el: ElementRef<'_>) -> String {
	let mut string = String::new();

	let mut t = el.text();
	loop {
		let Some(text) = t.next() else {
			break
		};

		if string.is_empty() {
			string += text.trim();
		} else {
			string += " ";
			string += text.trim();
		}
	}

	string.trim().to_string()
}

fn get_text_opt(el: ElementRef<'_>) -> Option<String> {
	let text = get_text(el);
	if text.is_empty() {
		None
	} else {
		Some(text)
	}
}

impl Financials {
	pub fn remove_duplicate(&mut self) {
		let dup = self.annuals.list.clone();
		for annual in dup.into_iter() {
			if let Some((i, fi)) = self.annuals.list.iter().enumerate().find(|(_, fi)| fi.year_month.year == annual.year_month.year) {
				// drops the older one
				if fi.year_month < annual.year_month {
					self.annuals.list.remove(i);
				}
			}
		}
	}
}
