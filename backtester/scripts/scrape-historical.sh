#!/bin/bash
START_DATE=$(date -v -365d +%F)
END_DATE=$(date +%F)
cargo run -- scrape polygon SPY 1 "$START_DATE 00:00:00" "$END_DATE 00:00:00"
cargo run -- scrape polygon SPY 5 "$START_DATE 00:00:00" "$END_DATE 00:00:00"
cargo run -- scrape polygon SPY 15 "$START_DATE 00:00:00" "$END_DATE 00:00:00"
