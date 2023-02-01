#!/bin/bash
START_DATE=$(date -v -400d +%F)
END_DATE=$(date +%F)
cargo run --release -- backtest polygon supertrend SPY 1 "START_DATE $END_DATE
