# stocks-rs
Monorepo for various projects related to stocks/backtesting/algorithmic trading/technical analysis

## How to use

```shell
# scrape candles from API/CSV -> SQLite
cargo run --release --bin scraper alpha_vantage 2023-01-01 2023-03-03
# hyperoptimizer technical analysis indicators parameters while scoring for profit/loss
cargo run --release --bin backtester alpha_vantage SPY 1 2023-01-01 2023-03-03
```
