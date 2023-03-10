#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! chrono = "0.4.23"
//! chrono-tz = "0.8.1"
//! csv = "1.2.1"
//! serde = { version = "1.0.153", features = ["derive"] }
//! ta = "0.5.0"
//! ```

use std::{collections::HashMap, fs::File};

use chrono::{TimeZone, Datelike, DateTime, NaiveDateTime, Weekday, Duration};
use chrono_tz::{US, Tz};
use csv::ReaderBuilder;
use serde::{Serialize, Deserialize};
use ta::{indicators::SimpleMovingAverage, Next};

#[derive(PartialEq, Debug)]
enum Direction {
  Long,
  Short,
  Flat,
}

#[derive(PartialEq, Debug)]
enum MarketSessionType {
  None,
  Pre,
  Regular,
  Post,
}

fn datetime_from_timestamp(timestamp: i64) -> DateTime<Tz> {
  let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
  return US::Eastern.from_utc_datetime(&naive);
}

fn get_regular_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = US::Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let end = US::Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (start, end);
}

fn determine_session_type(eastern_now: DateTime<Tz>) -> MarketSessionType {
  // short circuit on weekends
  let weekday = eastern_now.weekday();
  let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
  if is_weekend {
    return MarketSessionType::None;
  }
  // short circuit on holidays
  let holidays_2022 = vec![
    "2022-01-17 00:00:00", // martin luther king jr day
    "2022-02-21 00:00:00", // preisdent's day
    "2022-04-15 00:00:00", // good friday
    "2022-05-30 00:00:00", // memorial day
    "2022-06-20 00:00:00", // juneteenth
    "2022-07-04 00:00:00", // independence day
    "2022-09-05 00:00:00", // labor day
    "2022-11-24 00:00:00", // day before thanksgiving
    "2022-11-25 00:00:00", // day after thanksgiving (closes at 1pm)?
    "2022-12-26 00:00:00", // day after christmas
  ];
  let holidays_2023 = vec![
    "2023-01-02 00:00:00", // new year's day
    "2023-01-16 00:00:00", // martin luther king jr day
    "2023-02-20 00:00:00", // preisdent's day
    "2023-04-07 00:00:00", // good friday
    "2023-05-29 00:00:00", // memorial day
    "2023-06-19 00:00:00", // juneteenth
    "2023-07-04 00:00:00", // independence day
    "2023-09-04 00:00:00", // labor day
    "2023-11-23 00:00:00", // thanksgiving day
    "2023-11-24 00:00:00", // day after thanksgiving (closes at 1pm)?
    "2023-12-25 00:00:00", // christmas
  ];
  let formatted_eastern_now = eastern_now.format("%Y-%m-%d 00:00:00").to_string();
  let is_2022_holiday = holidays_2022.iter().any(|&holiday| holiday == formatted_eastern_now);
  let is_2023_holiday = holidays_2023.iter().any(|&holiday| holiday == formatted_eastern_now);
  let is_holiday = is_2022_holiday || is_2023_holiday;
  if is_holiday {
    return MarketSessionType::None;
  }
  // check pre/regular/post
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  // premarket: 4am -> 9:29:59am
  let pre_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 4, 0, 0).unwrap();
  let pre_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 9, 29, 59).unwrap();
  let seconds_before_pre_market = eastern_now.signed_duration_since(pre_market_start).num_seconds();
  let seconds_after_pre_market = eastern_now.signed_duration_since(pre_market_end).num_seconds();
  let is_before_pre_market = seconds_before_pre_market < 0;
  let is_after_pre_market = seconds_after_pre_market >= 0;
  let is_during_pre_market = is_before_pre_market == false && is_after_pre_market == false;
  // regular: 9:30am -> 3:59:59pm
  let regular_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap();
  let regular_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap();
  let seconds_before_regular_market = eastern_now.signed_duration_since(regular_market_start).num_seconds();
  let seconds_after_regular_market = eastern_now.signed_duration_since(regular_market_end).num_seconds();
  let is_before_regular_market = seconds_before_regular_market < 0;
  let is_after_regular_market = seconds_after_regular_market >= 0;
  let is_during_regular_market = is_before_regular_market == false && is_after_regular_market == false;
  // aftermarket: 4:00pm -> 7:59:59pm
  let after_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 16, 0, 0).unwrap();
  let after_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 19, 59, 59).unwrap();
  let seconds_before_after_market = eastern_now.signed_duration_since(after_market_start).num_seconds();
  let seconds_after_after_market = eastern_now.signed_duration_since(after_market_end).num_seconds();
  let is_before_after_market = seconds_before_after_market < 0;
  let is_after_after_market = seconds_after_after_market >= 0;
  let is_during_after_market = is_before_after_market == false && is_after_after_market == false;
  if is_during_pre_market {
    return MarketSessionType::Pre;
  } else if is_during_regular_market {
    return MarketSessionType::Regular;
  } else if is_during_after_market {
    return MarketSessionType::Post;
  } else {
    return MarketSessionType::None;
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Candle {
  pub start_timestamp: i64,
  pub end_timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

fn read_records_from_csv<T>(filename: &str) -> Vec<T>
where
  T: for<'de> Deserialize<'de>{
  let mut candles = vec![];
  let file = File::open(filename).unwrap();
  let mut csv_reader = ReaderBuilder::new()
    .has_headers(true)
    .from_reader(file);
  for record in csv_reader.deserialize() {
    let candle: T = record.unwrap();
    candles.push(candle);
  }
  return candles;
}

fn main() {
  // load candles
  let candles_filename = std::env::args().nth(1).unwrap();
  let candles = read_records_from_csv::<Candle>(&candles_filename);
  let mut candles_map = HashMap::new();
  for candle in &candles {
    candles_map.insert(candle.start_timestamp, candle);
  }
  // build indicators
  let fast_periods = 50;
  let slow_periods = 200;
  let mut fast = SimpleMovingAverage::new(fast_periods).unwrap();
  let mut slow = SimpleMovingAverage::new(slow_periods).unwrap();
  let mut last_fast = 0.0;
  let mut last_slow = 0.0;
  let mut num_periods = 0;
  // print header
  println!("start_timestamp,end_timestamp,direction");
  // traverse time
  let parsed_start = datetime_from_timestamp(candles[0].start_timestamp);
  let parsed_end = datetime_from_timestamp(candles[candles.len() - 1].end_timestamp);
  let mut pointer = parsed_start;
  while pointer.timestamp() <= parsed_end.timestamp() {
    let current_session_type = determine_session_type(pointer);
    // skip when market is not open
    if current_session_type == MarketSessionType::None {
      pointer = pointer + Duration::minutes(5);
      continue;
    }
    // get candle
    let candle = candles_map.get(&pointer.timestamp());
    if candle.is_none() {
      if current_session_type == MarketSessionType::Pre || current_session_type == MarketSessionType::Post {
        //println!("no candle for {pointer} {timestamp}?", timestamp = pointer.timestamp());
      }
      if current_session_type == MarketSessionType::Regular {
        panic!("no candle for {pointer} {timestamp}?", timestamp = pointer.timestamp());
      }
      pointer = pointer + Duration::minutes(5);
      continue;
    }
    let candle = candle.unwrap();
    // feed to indicators
    let hlc3 = (candle.high + candle.low + candle.close) / 3.0;
    last_fast = fast.next(hlc3);
    last_slow = slow.next(hlc3);
    num_periods += 1;
    // calculate warmup
    let is_warmed_up = num_periods >= slow_periods;
    // calculate direction
    let is_pre_market = current_session_type == MarketSessionType::Pre;
    let is_post_market = current_session_type == MarketSessionType::Post;
    let (_regular_session_start, regular_session_end) = get_regular_market_session_start_and_end(&pointer);
    let distance_to_regular_session_end = regular_session_end.timestamp() - pointer.timestamp();
    let candle_size_seconds = candle.end_timestamp - candle.start_timestamp + 1;
    let is_last_candle_of_regular_session = current_session_type == MarketSessionType::Regular && distance_to_regular_session_end <= (candle_size_seconds - 1);
    let should_be_flat = is_warmed_up == false || is_pre_market || is_post_market || is_last_candle_of_regular_session;
    let direction = if should_be_flat {
      Direction::Flat
    } else {
      if last_fast > last_slow {
        Direction::Long
      } else {
        Direction::Short
      }
    };
    // log
    println!("{start_timestamp},{end_timestamp},{direction:?}", start_timestamp = candle.start_timestamp, end_timestamp = candle.end_timestamp);
    // increment
    pointer = pointer + Duration::minutes(5);
  }
}
