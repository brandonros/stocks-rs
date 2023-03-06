use std::{collections::HashMap};

use crate::{candles, database::*, market_session, structs::*};

pub fn build_candles_date_map(connection: &Database, symbol: &str, resolution: &str, dates: &Vec<String>) -> HashMap<String, Vec<Candle>> {
  let mut candles_date_map = HashMap::new();
  for date in dates {
    //let (start, end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let (start, end) = market_session::get_extended_market_session_start_and_end_from_string(date);
    let start_timestamp = start.timestamp();
    let end_timestamp = end.timestamp();
    // get candles from database
    let candle_snapshots =
      candles::get_candle_snapshots_from_database(&connection, symbol, resolution, start_timestamp, end_timestamp);
    let candles: Vec<Candle> = candle_snapshots
      .iter()
      .map(|candle_snapshot| {
        return Candle {
          timestamp: candle_snapshot.timestamp,
          open: candle_snapshot.open,
          high: candle_snapshot.high,
          low: candle_snapshot.low,
          close: candle_snapshot.close,
          volume: candle_snapshot.volume as i64,
        };
      })
      .collect();
    let mut date_candles: Vec<Candle> = vec![];
    for candle in candles {
      date_candles.push(candle.clone());
    }
    assert_eq!(date_candles.len(), 390);
    candles_date_map.insert(date.clone(), date_candles);
  }
  return candles_date_map;
}
