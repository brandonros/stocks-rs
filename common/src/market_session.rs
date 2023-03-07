use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike};
use chrono_tz::{Tz, US::Eastern};

#[derive(PartialEq)]
pub enum MarketSessionType {
  None,
  Pre,
  Regular,
  Post,
}

pub fn determine_session_type(eastern_now: DateTime<Tz>) -> MarketSessionType {
  // short circuit pre-market / regular / after hours / weekends
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let weekday = eastern_now.weekday();
  // premarket: 4am -> 9:29:59am
  let pre_market_start = Eastern.with_ymd_and_hms(year, month, day, 4, 0, 0).unwrap();
  let pre_market_end = Eastern.with_ymd_and_hms(year, month, day, 9, 29, 59).unwrap();
  let seconds_before_pre_market = eastern_now.signed_duration_since(pre_market_start).num_seconds();
  let seconds_after_pre_market = eastern_now.signed_duration_since(pre_market_end).num_seconds();
  let is_before_pre_market = seconds_before_pre_market < 0;
  let is_after_pre_market = seconds_after_pre_market >= 0;
  let is_during_pre_market = is_before_pre_market == false && is_after_pre_market == false;
  // regular: 9:30am -> 3:59:59pm
  let regular_market_start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap();
  let regular_market_end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap();
  let seconds_before_regular_market = eastern_now.signed_duration_since(regular_market_start).num_seconds();
  let seconds_after_regular_market = eastern_now.signed_duration_since(regular_market_end).num_seconds();
  let is_before_regular_market = seconds_before_regular_market < 0;
  let is_after_regular_market = seconds_after_regular_market >= 0;
  let is_during_regular_market = is_before_regular_market == false && is_after_regular_market == false;
  // aftermarket: 4:00pm -> 7:59:59pm
  let after_market_start = Eastern.with_ymd_and_hms(year, month, day, 16, 0, 0).unwrap();
  let after_market_end = Eastern.with_ymd_and_hms(year, month, day, 19, 59, 59).unwrap();
  let seconds_before_after_market = eastern_now.signed_duration_since(after_market_start).num_seconds();
  let seconds_after_after_market = eastern_now.signed_duration_since(after_market_end).num_seconds();
  let is_before_after_market = seconds_before_after_market < 0;
  let is_after_after_market = seconds_after_after_market >= 0;
  let is_during_after_market = is_before_after_market == false && is_after_after_market == false;
  // weekend
  let is_weekend = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;
  if is_weekend {
    return MarketSessionType::None;
  } else if is_during_pre_market {
    return MarketSessionType::Pre;
  } else if is_during_regular_market {
    return MarketSessionType::Regular;
  } else if is_during_after_market {
    return MarketSessionType::Post;
  } else {
    return MarketSessionType::None;
  }
}

pub fn get_extended_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = Eastern.with_ymd_and_hms(year, month, day, 7, 0, 0).unwrap(); // 7:00:00am
  let end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm // TODO: 8pm?
  return (start, end);
}

pub fn get_extended_market_session_start_and_end_from_string(input: &str) -> (DateTime<Tz>, DateTime<Tz>) {
  let parsed_input = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
  let eastern_input = Eastern.from_local_datetime(&parsed_input).unwrap();
  return get_extended_market_session_start_and_end(&eastern_input);
}

pub fn get_regular_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (start, end);
}

pub fn get_regular_market_session_start_and_end_from_string(input: &str) -> (DateTime<Tz>, DateTime<Tz>) {
  let parsed_input = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
  let eastern_input = Eastern.from_local_datetime(&parsed_input).unwrap();
  return get_regular_market_session_start_and_end(&eastern_input);
}

pub fn get_current_candle_start_and_stop(resolution: &str, eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  // TODO: support resolutions other than 1 minute
  assert_eq!(resolution, "1");
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let hour = eastern_now.hour();
  let minute = eastern_now.minute();
  let start = Eastern.with_ymd_and_hms(year, month, day, hour, minute, 0).unwrap();
  let end = Eastern.with_ymd_and_hms(year, month, day, hour, minute, 59).unwrap();
  return (start, end);
}
