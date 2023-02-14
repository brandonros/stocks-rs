use std::collections::HashMap;

use crate::{database::*, structs::*};

pub fn get_candle_snapshots_from_database(
  connection: &Database,
  symbol: &str,
  resolution: &str,
  regular_market_start_timestamp: i64,
  regular_market_end_timestamp: i64,
) -> Vec<CandleSnapshot> {
  let query = format!(
    "
    select scraped_at,
      timestamp, 
      open, 
      high, 
      low,
      close,
      volume
    from candles 
    where timestamp >= {regular_market_start_timestamp} and timestamp <= {regular_market_end_timestamp}
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
    "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

pub fn get_live_candle_snapshots_from_database(
  connection: &Database,
  symbol: &str,
  resolution: &str,
  eastern_now_timestamp: i64,
  regular_market_start_timestamp: i64,
  candle_lookup_max_timestamp: i64,
) -> Vec<CandleSnapshot> {
  let query = format!(
    "
    select scraped_at,
      timestamp, 
      open, 
      high, 
      low,
      close,
      volume
    from candles 
    where timestamp >= {regular_market_start_timestamp} and timestamp <= {candle_lookup_max_timestamp}
    and scraped_at = (select scraped_at from candles where scraped_at >= {regular_market_start_timestamp} and scraped_at <= {eastern_now_timestamp} order by scraped_at desc limit 1) 
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
  "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

pub fn get_candles_by_date_as_continuous_vec(dates: &Vec<String>, candles_date_map: &HashMap<String, Vec<Candle>>) -> Vec<Candle> {
  let mut candles = vec![];
  for date in dates {
    let mut date_candles = candles_date_map.get(date).unwrap().clone();
    candles.append(&mut date_candles);
  }
  return candles;
}
