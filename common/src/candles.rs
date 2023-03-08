use std::collections::HashMap;
use ordered_float::OrderedFloat;

use crate::{structs::*};

pub fn get_candles_by_date_as_continuous_vec(dates: &Vec<String>, candles_date_map: &HashMap<String, Vec<Candle>>) -> Vec<Candle> {
  let mut candles = vec![];
  for date in dates {
    let mut date_candles = candles_date_map.get(date).unwrap().clone();
    let num_date_candles = date_candles.len();
    if num_date_candles < 108 {
      log::warn!("not enough candles? date = {date} num_date_candles = {num_date_candles}")
    }
    candles.append(&mut date_candles);
  }
  return candles;
}

pub fn convert_timeframe(start_timestamp: i64, end_timestamp: i64, resolution: &str, one_minute_candles: &Vec<Candle>) -> Vec<Candle> {
  let mut grouped_candles: Vec<Candle> = vec![];
  let mut pointer = start_timestamp;
  let step = if resolution == "1" {
    60
  } else if resolution == "5" {
    60 * 5
  } else {
    unimplemented!()
  };
  // loop over the entire trading day, grouping candles
  while pointer <= end_timestamp {
    let scaled_candle_start = pointer;
    let scaled_candle_end = pointer + step - 1;
    let scaled_candle_rows = one_minute_candles.iter().filter(|candle| {
      return candle.timestamp >= scaled_candle_start && candle.timestamp <= scaled_candle_end;
    }).collect::<Vec<_>>();
    if scaled_candle_rows.len() == 0 {
      log::warn!("no candles = {}", pointer);
      pointer += step;
      continue;
    }
    let first_row = &scaled_candle_rows[0];
    let last_row = &scaled_candle_rows[scaled_candle_rows.len() - 1]; // edge case example: pre-market doesn't always have 5 1 minute candles
    let timestamp = first_row.timestamp;
    let open = first_row.open;
    let low = scaled_candle_rows.iter().map(|candle| OrderedFloat(candle.low)).min().unwrap().into_inner();
    let high = scaled_candle_rows.iter().map(|candle| OrderedFloat(candle.high)).max().unwrap().into_inner();
    let close = last_row.close;
    let volume = scaled_candle_rows.iter().fold(0, |prev, candle| prev + candle.volume);
    grouped_candles.push(Candle {
      timestamp,
      open,
      high,
      low,
      close,
      volume
    });
    pointer += step;
  }
  return grouped_candles;
}
