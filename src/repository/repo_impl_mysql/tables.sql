SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS item_info;
SET FOREIGN_KEY_CHECKS = 1;
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

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS item_price;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE item_price (
	code VARCHAR(9) NOT NULL, -- 종목코드
	ord_date DATE NOT NULL, -- 영업일자
	closing INT UNSIGNED, -- 종가
	opening INT UNSIGNED, -- 시가
	highest INT UNSIGNED, -- 고가
	lowest INT UNSIGNED, -- 저가
	diff INT, -- 전일대비
	PRIMARY KEY(code, ord_date)
);

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS item_latest;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE item_latest (
	code VARCHAR(9) NOT NULL PRIMARY KEY, -- 종목코드
	ord_date DATE NOT NULL, -- 영업일자
	closing INT UNSIGNED, -- 종가
	opening INT UNSIGNED, -- 시가
	highest INT UNSIGNED, -- 고가
	lowest INT UNSIGNED, -- 저가
	diff INT, -- 전일대비
);

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS index_price;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE index_price (
	code VARCHAR(9) NOT NULL, -- 지수코드
	ord_date DATE NOT NULL, -- 영업일자
	closing DECIMAL(6,2) CHECK (closing >= 0), -- 종가
	opening DECIMAL(6,2) CHECK (opening >= 0), -- 시가
	highest DECIMAL(6,2) CHECK (highest >= 0), -- 고가
	lowest DECIMAL(6,2) CHECK (lowest >= 0), -- 저가
	diff DECIMAL(6,2), -- 전일대비
	PRIMARY KEY(code, ord_date)
);

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS funds;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE funds (
	id INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT, -- 펀드ID
	name VARCHAR(256) NOT NULL, -- 펀드명
	list_date DATE, -- 상장일자
	res_drv_run TINYINT UNSIGNED NOT NULL, -- 데이터 수집 여부
	res_drv_name VARCHAR(32), -- 데이터 수집 드라이버명
	res_drv_params JSON -- 데이터 수집 드라이버 파라미터
);

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS fund_price;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE fund_price (
	fund_id INT UNSIGNED NOT NULL, -- 펀드ID
	ord_date DATE NOT NULL, -- 영업일자
	std_price DECIMAL(6,2) NOT NULL CHECK (std_price >= 0), -- 기준가
	diff DECIMAL(6,2) NOT NULL, -- 전일대비
	input_size DOUBLE, -- 설정원본
	stock_size DOUBLE, -- 주식자산 금액
	bond_size DOUBLE, -- 채권자산 금액
	liquid_size DOUBLE, -- 유동자산 금액
	adj_price DECIMAL(8,2) CHECK (adj_price >= 0), -- 수정기준가
	PRIMARY KEY(fund_id, ord_date)
);

SET FOREIGN_KEY_CHECKS = 0;
DROP TABLE IF EXISTS fund_price_reset;
SET FOREIGN_KEY_CHECKS = 1;
CREATE TABLE fund_price_reset (
	fund_id INT UNSIGNED NOT NULL, -- 펀드ID
	ord_date DATE NOT NULL, -- 일자
	before_price DECIMAL(6,4) CHECK (before_price >= 0) NOT NULL, -- 초기화 이전 기준가
	after_price DECIMAL(6,4) CHECK (after_price >= 0) NOT NULL, -- 초기화 이후 기준가
	PRIMARY KEY(fund_id, ord_date)
);
