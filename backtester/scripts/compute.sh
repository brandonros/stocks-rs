#!/bin/bash
DATE=$(date +%F)
cargo run -- compute yahoo_finance supertrend SPY 1 "$DATE 00:00:00"
