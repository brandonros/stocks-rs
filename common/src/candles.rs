use std::collections::HashMap;
use ordered_float::OrderedFloat;

use crate::{database::*, structs::*};

pub fn get_candle_snapshots_from_database(
  connection: &Database,
  symbol: &str,
  resolution: &str,
  start_timestamp: i64,
  end_timestamp: i64,
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
    where timestamp >= {start_timestamp} and timestamp <= {end_timestamp}
    and scraped_at = (select scraped_at from candles where timestamp >= {start_timestamp} and timestamp <= {end_timestamp} order by scraped_at desc limit 1) 
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
  start_timestamp: i64,
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
    where timestamp >= {start_timestamp} and timestamp <= {candle_lookup_max_timestamp}
    and scraped_at = (select scraped_at from candles where scraped_at >= {start_timestamp} and scraped_at <= {eastern_now_timestamp} order by scraped_at desc limit 1) 
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
    let num_date_candles = date_candles.len();
    if num_date_candles < 500 {
      log::warn!("not enough candles? date = {date} num_date_candles = {num_date_candles}")
    }
    candles.append(&mut date_candles);
  }
  return candles;
}

pub fn convert_timeframe(candles: &Vec<Candle>, source_timeframe: usize, target_timeframe: usize) -> Vec<Candle> {
  assert!(source_timeframe == 1);
  let chunks: Vec<&[Candle]> = candles.chunks(target_timeframe).collect();
  return chunks.into_iter().map(|chunk| {
    // check length
    if chunk.len() < target_timeframe {
      panic!("not enough candles {:?}", chunk);
    }
    let timestamp = chunk[0].timestamp;
    let open = chunk[0].open;
    let low = chunk.iter().map(|candle| OrderedFloat(candle.low)).min().unwrap().into_inner();
    let high = chunk.iter().map(|candle| OrderedFloat(candle.high)).max().unwrap().into_inner();
    let close = chunk[target_timeframe - 1].close;
    let volume = chunk.iter().fold(0, |prev, candle| prev + candle.volume);
    return Candle {
      timestamp,
      open,
      high,
      low,
      close,
      volume
    };
  }).collect();
}
