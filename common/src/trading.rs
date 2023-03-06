use std::{collections::{HashMap}};

use chrono::DateTime;
use chrono_tz::Tz;
use ta::{Next};

use crate::{market_session, structs::*, dates};

fn calculate_vwap_zscore_direction(trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Direction {
  // mean = sum(volume*close,pds)/sum(volume,pds)
  let periods = trade_generation_context.sma_periods;
  let most_recent_candles = candles.as_slice()[candles.len()-periods..].to_vec();
  let mean_dividend: f64 = most_recent_candles.iter().map(|candle| candle.volume as f64 * candle.close).sum();
  let mean_divisor: f64 = most_recent_candles.iter().map(|candle| candle.volume as f64).sum();
  let mean = mean_dividend / mean_divisor;
  // vwapsd = sqrt(sma(pow(close-mean, 2), pds) )
  let mut indicator = ta::indicators::SimpleMovingAverage::new(periods).unwrap();
  let mut last_sma = 0.0;
  for candle in candles {
    last_sma = indicator.next((candle.close - mean).powf(2.0));
  }
  let vwapsd = last_sma.sqrt();
  // (close-mean)/vwapsd
  let most_recent_close = candles[candles.len() - 1].close;
  let z_distance = (most_recent_close - mean) / vwapsd;
  // oversold
  if z_distance <= (trade_generation_context.oversold_z_distance * -1.0) {
    return Direction::Long;
  }
  // overbought
  if z_distance >= trade_generation_context.overbought_z_distance {
    return Direction::Short;
  }
  return Direction::Flat;
}

/*
fn pine_sma(values: &Vec<f64>, periods: usize) -> Vec<f64> {
  let mut indicator = ta::indicators::SimpleMovingAverage::new(periods).unwrap();
  let mut results = vec![];
  for value in values {
    results.push(indicator.next(*value));
  }
  return results;
}
fn calculate_fair_value_direction(trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Direction {
  let sma_periods = trade_generation_context.sma_periods;
  let median_up_deviation = trade_generation_context.median_up_deviation;
  let median_down_deviation = trade_generation_context.median_down_deviation;
  let band_boost = trade_generation_context.band_boost;
  let ohlc4s: Vec<f64> = candles.iter().map(|candle| {
    return (candle.open + candle.high + candle.low + candle.close) / 4.0;
  }).collect();
  let ohlc4_sma = pine_sma(&ohlc4s, sma_periods);
  let fair_price_smooth = ohlc4_sma[ohlc4_sma.len() - 1];
  let upper_band = fair_price_smooth * median_up_deviation;
  let lower_band = fair_price_smooth * median_down_deviation;
  let band_up_spread = upper_band - fair_price_smooth;
  let band_down_spread = fair_price_smooth - lower_band;
  let upper_band_boosted = fair_price_smooth + (band_up_spread * band_boost);
  let lower_band_boosted = fair_price_smooth - (band_down_spread * band_boost);
  let most_recent_candle = &candles[candles.len() - 1];
  let trend_rule_up = most_recent_candle.low > upper_band_boosted;
  let trend_rule_down = most_recent_candle.high < lower_band_boosted;
  if trend_rule_down {
    return Direction::Short;
  }
  if trend_rule_up {
    return Direction::Long;
  }
  return Direction::Flat;
}
*/

fn calculate_trades_from_direction_snapshots(direction_snapshots: &Vec<DirectionSnapshot>) -> Vec<Trade> {
  let mut buckets: Vec<Vec<DirectionSnapshot>> = Vec::new();
  let mut bucket: Vec<DirectionSnapshot> = Vec::new();
  let mut current_direction = Direction::Flat;
  for direction_snapshot in direction_snapshots {
    let should_open_new_bucket = match (current_direction, direction_snapshot.direction) {
      // stay in (no change)
      (Direction::Short, Direction::Short) => {
        false
      }
      (Direction::Long, Direction::Long) => {
        false
      }
      (Direction::Flat, Direction::Flat) => {
        false
      }
      // stay in (crossover/crossunder once)
      (Direction::Long, Direction::Flat) => {
        false
      }
      (Direction::Short, Direction::Flat) => {
        false
      }
      // open new
      (Direction::Flat, Direction::Long) => {
        true
      }
      (Direction::Flat, Direction::Short) => {
        true
      }
      // switch direction
      (Direction::Short, Direction::Long) => {
        true
      }
      (Direction::Long, Direction::Short) => {
        true
      }
    };
    if should_open_new_bucket {
      buckets.push(bucket);
      bucket = Vec::new();
      current_direction = direction_snapshot.direction;
    }
    bucket.push(direction_snapshot.clone());
  }
  if bucket.is_empty() == false {
    buckets.push(bucket);
  }
  return buckets
    .iter()
    .filter(|bucket| {
      return bucket.is_empty() == false;
    })
    .map(|bucket| {
      return Trade {
        start_timestamp: bucket[0].timestamp,
        end_timestamp: bucket[bucket.len() - 1].timestamp,
        formatted_start_timestamp: dates::format_timestamp(bucket[0].timestamp),
        formatted_end_timestamp: dates::format_timestamp(bucket[bucket.len() - 1].timestamp),
        direction: bucket[0].direction,
      };
    })
    .filter(|bucket| {
      return bucket.direction != Direction::Flat;
    })
    .collect();
}

fn generate_direction_snapshots(
  trade_generation_context: &TradeGenerationContext,
  start: DateTime<Tz>,
  end: DateTime<Tz>,
  candles: &Vec<Candle>,
) -> Vec<DirectionSnapshot> {
  let mut pointer = start;
  let mut direction_snapshots: Vec<DirectionSnapshot> = vec![];
  // iterate over every minute of the trading day, making sure we do not include the end of the most recent candle because it would not be known in a live situation
  while pointer <= end {
    let reduced_candles: Vec<Candle> = candles
      .iter()
      .cloned()
      .filter(|candle| return candle.timestamp < pointer.timestamp())
      .collect();
    // allow warmup
    if reduced_candles.len() < trade_generation_context.warmup_periods {
      pointer += chrono::Duration::minutes(1);
      continue;
    }
    // calculate direction
    let direction = calculate_vwap_zscore_direction(trade_generation_context, &reduced_candles);
    direction_snapshots.push(DirectionSnapshot {
      timestamp: pointer.timestamp(),
      direction,
    });
    pointer += chrono::Duration::minutes(1);
  }
  return direction_snapshots;
}

pub fn generate_dates_trades_map(
  dates: &Vec<String>,
  trade_generation_context: &TradeGenerationContext,
  candles_date_map: &HashMap<String, Vec<Candle>>,
) -> HashMap<String, Vec<Trade>> {
  let mut dates_trades_map = HashMap::new();
  for date in dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let direction_snapshots = generate_direction_snapshots(&trade_generation_context, regular_market_start, regular_market_end, date_candles);
    if direction_snapshots.is_empty() {
      //log::warn!("date = {} direction_snapshots.is_empty()", date);
      dates_trades_map.insert(date.clone(), vec![]);
      continue;
    }
    let date_trades = calculate_trades_from_direction_snapshots(&direction_snapshots);
    //log::info!("date = {} num_trades = {}", date, date_trades.len());
    dates_trades_map.insert(date.clone(), date_trades);
  }
  return dates_trades_map;
}

pub fn generate_continuous_trades(dates: &Vec<String>, trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Vec<Trade> {
  let mut direction_snapshots = vec![];
  for date in dates {
    let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let mut date_direction_snapshots = generate_direction_snapshots(&trade_generation_context, regular_market_start, regular_market_end, &candles);
    direction_snapshots.append(&mut date_direction_snapshots);
  }
  return calculate_trades_from_direction_snapshots(&direction_snapshots);
}
