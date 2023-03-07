use chrono::DateTime;
use chrono_tz::Tz;

use crate::{market_session, structs::*, dates, strategy};

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
      // stay in or stay out? (crossover/crossunder once)
      (Direction::Long, Direction::Flat) => {
        true // TODO: close on long -> flat or stay open?
      }
      (Direction::Short, Direction::Flat) => {
        true // TODO: close on Short -> flat or stay open?
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
  let pointer_step = 1; // TODO: 1 or 5?
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
      pointer += chrono::Duration::minutes(pointer_step);
      continue;
    }
    // calculate direction
    let direction = strategy::calculate_direction_snapshot(start, end, pointer, &reduced_candles, trade_generation_context);
    direction_snapshots.push(DirectionSnapshot {
      timestamp: pointer.timestamp(),
      direction,
    });
    // log
    let most_recent_candle = &reduced_candles[reduced_candles.len() - 1];
    log::trace!("{},{:?},{},{},{},{},{}", pointer.timestamp(), direction, most_recent_candle.open, most_recent_candle.high, most_recent_candle.low, most_recent_candle.close, most_recent_candle.volume);
    // increment
    pointer += chrono::Duration::minutes(pointer_step);
  }
  return direction_snapshots;
}

pub fn generate_continuous_trades(dates: &Vec<String>, trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Vec<Trade> {
  let mut direction_snapshots = vec![];
  for date in dates {
    // TODO: regular or extended?
    //let (start, end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let (start, end) = market_session::get_extended_market_session_start_and_end_from_string(date);
    let mut date_direction_snapshots = generate_direction_snapshots(&trade_generation_context, start, end, &candles);
    direction_snapshots.append(&mut date_direction_snapshots);
  }
  return calculate_trades_from_direction_snapshots(&direction_snapshots);
}
