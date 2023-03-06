use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike};
use chrono_tz::{Tz, US::Eastern};

pub fn get_extended_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = Eastern.with_ymd_and_hms(year, month, day, 7, 0, 0).unwrap(); // 7:00:00am
  let end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm // TODO: 8pm?
  return (start, end);
}

pub fn get_regular_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (start, end);
}

pub fn get_extended_market_session_start_and_end_from_string(input: &str) -> (DateTime<Tz>, DateTime<Tz>) {
  let parsed_input = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
  let eastern_input = Eastern.from_local_datetime(&parsed_input).unwrap();
  return get_extended_market_session_start_and_end(&eastern_input);
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
