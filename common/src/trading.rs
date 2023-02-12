use std::{collections::{HashMap}};

use ta::{Next};

use crate::{market_session, structs::*};

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
        direction: bucket[0].direction,
      };
    })
    .filter(|bucket| {
      return bucket.direction != Direction::Flat;
    })
    .collect();
}

/*
Z distance from VWAP [LazyBear]
calc_zvwap(pds) =>
	mean = sum(volume*close,pds)/sum(volume,pds)
	vwapsd = sqrt(sma(pow(close-mean, 2), pds) )
	(close-mean)/vwapsd
*/
fn calculate_direction(trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Direction {
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

fn generate_direction_snapshots(
  trade_generation_context: &TradeGenerationContext,
  date: &str,
  date_candles: &Vec<Candle>,
  strategy_name: &str,
) -> Vec<DirectionSnapshot> {
  assert!(strategy_name == "vwap_hlc3_divergence"); // TODO: more strategies?
  let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
  let mut pointer = regular_market_start;
  let mut direction_snapshots: Vec<DirectionSnapshot> = vec![];
  // iterate over every minute of the trading day, making sure we do not include the end of the most recent candle because it would not be known in a live situation
  while pointer <= regular_market_end {
    let reduced_candles: Vec<Candle> = date_candles
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
    let direction = calculate_direction(trade_generation_context, &reduced_candles);
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
  strategy_name: &str,
  candles_date_map: &HashMap<String, Vec<Candle>>,
) -> HashMap<String, Vec<Trade>> {
  let mut dates_trades_map = HashMap::new();
  for date in dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let direction_snapshots = generate_direction_snapshots(&trade_generation_context, date, date_candles, &strategy_name);
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
