#!/bin/bash
START_DATE=$(date +%F)
END_DATE=$(date +%F)
cargo run -- scrape polygon SPY 1 $START_DATE $END_DATE
cargo run -- scrape polygon SPY 5 $START_DATE $END_DATE
cargo run -- scrape polygon SPY 15 $START_DATE $END_DATE
