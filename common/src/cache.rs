use std::{collections::HashMap, sync::Arc};

use crate::{market_session, structs::*, database::*, candles};

pub fn build_candles_date_map(connection: &Database, symbol: &str, resolution: &str, dates: &Vec<String>) -> HashMap<String, Vec<Arc<Candle>>> {
  let mut candles_date_map = HashMap::new();
  for date in dates {
    let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let regular_market_start_timestamp = regular_market_start.timestamp();
    let regular_market_end_timestamp = regular_market_end.timestamp();
    // get candles from database
    let candle_snapshots = candles::get_candle_snapshots_from_database(&connection, symbol, resolution, regular_market_start_timestamp, regular_market_end_timestamp);
    let candles: Vec<Arc<Candle>> = candle_snapshots
      .iter()
      .map(|candle_snapshot| {
        return Arc::new(Candle {
          timestamp: candle_snapshot.timestamp,
          open: candle_snapshot.open,
          high: candle_snapshot.high,
          low: candle_snapshot.low,
          close: candle_snapshot.close,
          volume: candle_snapshot.volume as i64,
        });
      })
      .collect();
    let mut date_candles: Vec<Arc<Candle>> = vec![];
    for candle in candles {
      date_candles.push(candle.clone());
    }
    candles_date_map.insert(date.clone(), date_candles);
  }
  return candles_date_map;
}

