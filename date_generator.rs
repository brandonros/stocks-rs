#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! chrono = "0.4.0"
//! chrono-tz = "0.8.0"
//! ```

use chrono::{NaiveDateTime, TimeZone, DateTime, Weekday, Datelike};
use chrono_tz::{US, Tz};

#[derive(PartialEq)]
enum MarketSessionType {
  None,
  Pre,
  Regular,
  Post,
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

fn main() {
  let start_date = std::env::args().nth(1).unwrap();
  let end_date = std::env::args().nth(2).unwrap();
  let parsed_start = NaiveDateTime::parse_from_str(&format!("{start_date} 04:00:00"), "%Y-%m-%d %H:%M:%S").unwrap();
  let parsed_end = NaiveDateTime::parse_from_str(&format!("{end_date} 04:00:00"), "%Y-%m-%d %H:%M:%S").unwrap();
  let parsed_start = US::Eastern.from_local_datetime(&parsed_start).unwrap();
  let parsed_end = US::Eastern.from_local_datetime(&parsed_end).unwrap();
  let mut pointer = parsed_start.clone();
  while pointer <= parsed_end {
      let session_type = determine_session_type(pointer);
      if session_type != MarketSessionType::None {
          println!("{}", pointer.format("%Y-%m-%d"));
      }
      pointer += chrono::Duration::days(1);
  }
}
