#!/bin/bash
START_DATE=$(date -v -365d +%F)
END_DATE=$(date +%F)
cargo run --bin historical_candle_scraper polygon $START_DATE $END_DATE
