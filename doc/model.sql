CREATE TABLE item_info (
	code VARCHAR(9) NOT NULL PRIMARY KEY, -- 종목코드
	info_date DATE NOT NULL, -- 종목정보 기준일자
	name VARCHAR(50) NOT NULL, -- 계좌명
	market enum('KOSPI', 'KOSDAQ', 'ETF') NOT NULL, -- 시장구분
	std_code VARCHAR(12), -- 표준코드 (예: KR7391680006)
	list_date DATE, -- 상장일자
	kind enum('보통주', '구형우선주', '신형우선주', '종류주권'), -- 주식종류
	secu_group enum('주권', '투자회사', '부동산투자회사', '주식예탁증권', '"사회간접자본투융자회사', '선박투자회사', '외국주권'), -- 증권구분
	sect VARCHAR(20), -- 소속부
	par INT UNSIGNED, -- 액면가
	list_shares BIGINT UNSIGNED, -- 상장주식수
	etf_obj_idx VARCHAR(100), -- 기초지수명
	etf_idx_inst VARCHAR(50), -- 지수산출기관
	etf_idx_multiplier INT, -- 추적배수
	etf_replica_method VARCHAR(20), -- 복제방법
	etf_idx_market VARCHAR(20), -- 기초시장분류
	etf_idx_asset VARCHAR(20), -- 기초자산분류
	etf_op_company VARCHAR(50), -- 운용사
	etf_fee_rate DECIMAL(4,3) CHECK (etf_fee_rate >= 0), -- 총 보수
	etf_tax_type VARCHAR(30) -- 과세유형
);

DROP TABLE IF EXISTS fi_annuals;
CREATE TABLE fi_annuals (
	stock_code VARCHAR(9) NOT NULL, -- 종목코드
	year SMALLINT UNSIGNED NOT NULL, -- 기준년도
	month TINYINT UNSIGNED NOT NULL, -- 기준월
	sales FLOAT, -- 매출액
	profit FLOAT, -- 영업이익
	net_income FLOAT, -- 당기순이익
	dividend FLOAT, -- 주당배당금
	dividend_yield FLOAT, -- 배당수익률
	PRIMARY KEY (stock_code, year),
	CHECK (month BETWEEN 1 AND 12)
);

DROP TABLE IF EXISTS fi_quarters;
CREATE TABLE fi_quarters (
	stock_code VARCHAR(9) NOT NULL, -- 종목코드
	year SMALLINT UNSIGNED NOT NULL, -- 기준년도
	month TINYINT UNSIGNED NOT NULL, -- 기준월
	sales FLOAT, -- 매출액
	profit FLOAT, -- 영업이익
	net_income FLOAT, -- 당기순이익
	dividend FLOAT, -- 주당배당금
	dividend_yield FLOAT, -- 배당수익률
	PRIMARY KEY (stock_code, year, month),
	CHECK (month BETWEEN 1 AND 12)
);
