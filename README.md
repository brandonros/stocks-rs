# stocks-rs
Monorepo for various projects related to stocks/backtesting/algorithmic trading/technical analysis

## Architecture

```
candles -> strategy -> indicators -> signals -> trades
```

## Backtesting flow

```
for every date worth of candle data
  for every possible setting combination
    for every strategy indicator signal direction change
      determiine stop loss, profit limit, or direction change
aggregate statistics into win/loss ratio, profit/loss performance, etc.
```

## Historical backtesting

```shell
START_DATE=$(date -v -400d +%F)
END_DATE=$(date +%F)

cargo run --bin historical_candle_scraper polygon 2023-01-05 2023-01-05
cargo run --release --bin trade_generator polygon SPY 1 2023-02-10 2023-02-10 # outputs to /tmp/trades.json
cargo run --release --bin trade_backtester polygon SPY 1 2022-01-01 2023-02-10 # outputs to /tmp/trade-results.json
cargo run --release --bin combination_backtester polygon SPY 1 2023-02-13 2023-02-13 # one day
cargo run --release --bin combination_backtester polygon SPY 1 2023-01-01 2023-02-10 # ytd
cargo run --release --bin combination_backtester polygon SPY 1 2022-01-01 2023-02-10 # one year-ish
```
