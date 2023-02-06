use chrono::{Datelike, Days, NaiveDateTime, TimeZone, Weekday, DateTime};
use chrono_tz::{US::Eastern, Tz};

pub fn build_list_of_dates(from: &str, to: &str) -> Vec<String> {
  let parsed_from = NaiveDateTime::parse_from_str(from, "%Y-%m-%d %H:%M:%S").unwrap();
  let parsed_to = NaiveDateTime::parse_from_str(to, "%Y-%m-%d %H:%M:%S").unwrap();
  let from = Eastern.from_local_datetime(&parsed_from).unwrap();
  let to = Eastern.from_local_datetime(&parsed_to).unwrap();
  let mut results = vec![];
  let mut pointer = from;
  while pointer.timestamp() <= to.timestamp() {
    let datetime = Eastern.timestamp_opt(pointer.timestamp(), 0).unwrap();
    let is_weekend = datetime.weekday() == Weekday::Sat || datetime.weekday() == Weekday::Sun;
    if is_weekend {
      pointer = pointer.checked_add_days(Days::new(1)).unwrap();
      continue;
    }
    let formatted_timestamp = datetime.format("%Y-%m-%d 00:00:00").to_string();
    // TODO: holidays?
    /*
    ? - new year's day
    2022-01-16 - martin luther king jr day
    2022-02-21 - preisdent's day
    2022-04-15 - good friday
    2022-05-30 - memorial day
    2022-06-20 - juneteenth
    2022-07-04 - independence day
    2022-09-05 - labor day
    2022-11-24 - day before thanksgiving
    2022-11-25 - day after thanksgiving (closes at 1pm)
    2022-12-26 - day after christmas
    */
    if formatted_timestamp != "2022-11-25 00:00:00" {
      results.push(formatted_timestamp);
    }
    pointer = pointer.checked_add_days(Days::new(1)).unwrap();
  }
  return results;
}

pub fn datetime_from_timestamp(timestamp: i64) -> DateTime<Tz> {
  let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
  return chrono_tz::US::Eastern.from_utc_datetime(&naive);
}

pub fn format_timestamp(timestamp: i64) -> String {
  return datetime_from_timestamp(timestamp).format("%Y-%m-%d %I:%M:%S %p").to_string();
}
