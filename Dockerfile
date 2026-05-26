FROM rust:1.95-alpine AS builder
WORKDIR /app
#RUN apk add libressl-dev musl-dev
RUN apk add openssl-dev openssl-libs-static musl-dev

COPY repo-helper/src ./repo-helper/src
COPY repo-helper/Cargo.toml ./repo-helper/Cargo.toml

COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml ./.cargo/config.toml
COPY src ./src
RUN cargo br --no-default-features --features=mysql --bin=stock-dp-scrap

# --------------

FROM alpine

WORKDIR /app
COPY --from=builder /app/target/release/stock-dp-scrap ./stock-dp-scrap

ENV TZ=Asia/Seoul
RUN apk add --no-cache tzdata
RUN cp /usr/share/zoneinfo/$TZ /etc/localtime
RUN echo $TZ > /etc/timezone

ENV DATABASE_URL=mysql://sise:sise@172.17.0.1:3306/sise
ENV RUST_LOG=info,stock_fn_scraper=debug,stock_dp_scrap=debug

CMD [ "./stock-dp-scrap" ]
