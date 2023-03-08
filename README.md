# stocks-rs
Monorepo for various projects related to stocks/backtesting/algorithmic trading/technical analysis

## How to use

```shell
# scrape candles from API/CSV -> SQLite
cargo run --release --bin scraper alpha_vantage 2023-01-01 2023-03-03
cargo run --release --bin scraper polygon 2022-01-01 2023-03-06
# hyperoptimizer technical analysis indicators parameters while scoring for profit/loss
cargo run --release --bin backtester polygon SPY 1 2023-01-01 2023-03-03
cargo run --release --bin backtester alpha_vantage SPY 1 2023-01-01 2023-03-03
```

## Results

```
cargo run --release --bin backtester polygon SPY 1 01-01-2023 01-01-2023
cargo run --release --bin backtester polygon SPY 1 01-02-2023 01-02-2023
cargo run --release --bin backtester polygon SPY 1 01-03-2023 01-03-2023
cargo run --release --bin backtester polygon SPY 1 01-04-2023 01-04-2023
cargo run --release --bin backtester polygon SPY 1 01-05-2023 01-05-2023
cargo run --release --bin backtester polygon SPY 1 01-06-2023 01-06-2023
cargo run --release --bin backtester polygon SPY 1 01-07-2023 01-07-2023
cargo run --release --bin backtester polygon SPY 1 01-08-2023 01-08-2023
cargo run --release --bin backtester polygon SPY 1 01-09-2023 01-09-2023
cargo run --release --bin backtester polygon SPY 1 01-10-2023 01-10-2023
cargo run --release --bin backtester polygon SPY 1 01-11-2023 01-11-2023
cargo run --release --bin backtester polygon SPY 1 01-12-2023 01-12-2023
cargo run --release --bin backtester polygon SPY 1 01-13-2023 01-13-2023
cargo run --release --bin backtester polygon SPY 1 01-14-2023 01-14-2023
cargo run --release --bin backtester polygon SPY 1 01-15-2023 01-15-2023
cargo run --release --bin backtester polygon SPY 1 01-16-2023 01-16-2023
cargo run --release --bin backtester polygon SPY 1 01-17-2023 01-17-2023
cargo run --release --bin backtester polygon SPY 1 01-18-2023 01-18-2023
cargo run --release --bin backtester polygon SPY 1 01-19-2023 01-19-2023
cargo run --release --bin backtester polygon SPY 1 01-20-2023 01-20-2023
cargo run --release --bin backtester polygon SPY 1 01-21-2023 01-21-2023
cargo run --release --bin backtester polygon SPY 1 01-22-2023 01-22-2023
cargo run --release --bin backtester polygon SPY 1 01-23-2023 01-23-2023
cargo run --release --bin backtester polygon SPY 1 01-24-2023 01-24-2023
cargo run --release --bin backtester polygon SPY 1 01-25-2023 01-25-2023
cargo run --release --bin backtester polygon SPY 1 01-26-2023 01-26-2023
cargo run --release --bin backtester polygon SPY 1 01-27-2023 01-27-2023
cargo run --release --bin backtester polygon SPY 1 01-28-2023 01-28-2023
cargo run --release --bin backtester polygon SPY 1 01-29-2023 01-29-2023
cargo run --release --bin backtester polygon SPY 1 01-30-2023 01-30-2023
cargo run --release --bin backtester polygon SPY 1 01-31-2023 01-31-2023
cargo run --release --bin backtester polygon SPY 1 02-01-2023 02-01-2023
cargo run --release --bin backtester polygon SPY 1 02-02-2023 02-02-2023
cargo run --release --bin backtester polygon SPY 1 02-03-2023 02-03-2023
cargo run --release --bin backtester polygon SPY 1 02-04-2023 02-04-2023
cargo run --release --bin backtester polygon SPY 1 02-05-2023 02-05-2023
cargo run --release --bin backtester polygon SPY 1 02-06-2023 02-06-2023
cargo run --release --bin backtester polygon SPY 1 02-07-2023 02-07-2023
cargo run --release --bin backtester polygon SPY 1 02-08-2023 02-08-2023
cargo run --release --bin backtester polygon SPY 1 02-09-2023 02-09-2023
cargo run --release --bin backtester polygon SPY 1 02-10-2023 02-10-2023
cargo run --release --bin backtester polygon SPY 1 02-11-2023 02-11-2023
cargo run --release --bin backtester polygon SPY 1 02-12-2023 02-12-2023
cargo run --release --bin backtester polygon SPY 1 02-13-2023 02-13-2023
cargo run --release --bin backtester polygon SPY 1 02-14-2023 02-14-2023
cargo run --release --bin backtester polygon SPY 1 02-15-2023 02-15-2023
cargo run --release --bin backtester polygon SPY 1 02-16-2023 02-16-2023
cargo run --release --bin backtester polygon SPY 1 02-17-2023 02-17-2023
cargo run --release --bin backtester polygon SPY 1 02-18-2023 02-18-2023
cargo run --release --bin backtester polygon SPY 1 02-19-2023 02-19-2023
cargo run --release --bin backtester polygon SPY 1 02-20-2023 02-20-2023
cargo run --release --bin backtester polygon SPY 1 02-21-2023 02-21-2023
cargo run --release --bin backtester polygon SPY 1 02-22-2023 02-22-2023
cargo run --release --bin backtester polygon SPY 1 02-23-2023 02-23-2023
cargo run --release --bin backtester polygon SPY 1 02-24-2023 02-24-2023
cargo run --release --bin backtester polygon SPY 1 02-25-2023 02-25-2023
cargo run --release --bin backtester polygon SPY 1 02-26-2023 02-26-2023
cargo run --release --bin backtester polygon SPY 1 02-27-2023 02-27-2023
cargo run --release --bin backtester polygon SPY 1 02-28-2023 02-28-2023
cargo run --release --bin backtester polygon SPY 1 03-01-2023 03-01-2023
cargo run --release --bin backtester polygon SPY 1 03-02-2023 03-02-2023
cargo run --release --bin backtester polygon SPY 1 03-03-2023 03-03-2023
cargo run --release --bin backtester polygon SPY 1 03-04-2023 03-04-2023
cargo run --release --bin backtester polygon SPY 1 03-05-2023 03-05-2023
cargo run --release --bin backtester polygon SPY 1 03-06-2023 03-06-2023
```