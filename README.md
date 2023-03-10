# stocks-rs

## Prerequisites

```shell
cargo install mask
cargo install rust-script
```

## How to use

```shell
mkdir output/
mkdir data/
mask generate_dates
mask scrape_dates
mask transform
mask generate_signals
mask generate_trades
mask backtest_trades
```
