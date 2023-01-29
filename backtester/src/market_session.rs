use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone};
use chrono_tz::{Tz, US::Eastern};

pub fn get_regular_market_start_end_from_string(input: &str) -> (DateTime<Tz>, DateTime<Tz>) {
  let parsed_input = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").unwrap();
  let eastern_input = Eastern.from_local_datetime(&parsed_input).unwrap();
  let year = eastern_input.year();
  let month = eastern_input.month();
  let day = eastern_input.day();
  let regular_market_start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap();
  let regular_market_end = Eastern.with_ymd_and_hms(year, month, day, 16, 0, 0).unwrap(); // TODO: 3:59:59pm instead?
  return (regular_market_start, regular_market_end);
}
