use chrono::{DateTime, Datelike, TimeZone, NaiveDateTime};
use chrono_tz::{Tz, US::Eastern};

pub fn get_regular_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let regular_market_start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let regular_market_end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (regular_market_start, regular_market_end);
}

pub fn get_regular_market_session_start_and_end_from_string(input: &str) -> (DateTime<Tz>, DateTime<Tz>) {
  let parsed_input = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
  let eastern_input = Eastern.from_local_datetime(&parsed_input).unwrap();
  return get_regular_market_session_start_and_end(&eastern_input);
}
