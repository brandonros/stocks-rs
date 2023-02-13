use std::{collections::{HashMap}};

use ta::{Next, DataItem};

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

fn value_to_data_item(value: f64) -> DataItem {
  return DataItem::builder()
    .open(value)
    .high(value)
    .low(value)
    .close(value)
    .build()
    .unwrap();
}

fn candle_to_data_item(candle: &Candle) -> DataItem {
  return DataItem::builder()
    .open(candle.open)
    .high(candle.high)
    .low(candle.low)
    .close(candle.close)
    .volume(candle.volume as f64)
    .build()
    .unwrap();
}

fn pine_cci(candles: &Vec<Candle>, periods: usize) -> Vec<f64> {
  let mut indicator = ta::indicators::CommodityChannelIndex::new(periods).unwrap();
  let mut results = vec![];
  for candle in candles {
    results.push(indicator.next(&candle_to_data_item(candle)));
  }
  return results;
}

fn pine_sma(values: &Vec<f64>, periods: usize) -> Vec<f64> {
  let mut indicator = ta::indicators::SimpleMovingAverage::new(periods).unwrap();
  let mut results = vec![];
  for value in values {
    results.push(indicator.next(*value));
  }
  return results;
}

fn pine_stoch(values: &Vec<f64>, periods: usize) -> Vec<f64> {
  let mut indicator = ta::indicators::FastStochastic::new(periods).unwrap();
  let mut results = vec![];
  for value in values {
    results.push(indicator.next(*value));
  }
  return results;
}

/*
CCI Stochastic and a quick lesson on Scalping & Trading Systems by Daveatt 

source = input(close)
cci_period = input(28, "CCI Period")
stoch_period = input(28, "Stoch Period")
stoch_smooth_k = input(3, "Stoch Smooth K")
stoch_smooth_d = input(3, "Stoch Smooth D")
d_or_k = input(defval="D", options=["D", "K"])
OB = input(80, "Overbought", type=input.integer)
OS = input(20, "Oversold", type=input.integer)

stoch = 100 * (close - min(low, length)) / (max(high, length) - min(low, length)).
cci = cci(source, cci_period)
stoch_cci_k = sma(stoch(cci, cci, cci, stoch_period), stoch_smooth_k)
stoch_cci_d = sma(stoch_cci_k, stoch_smooth_d)

ma = (d_or_k == "D") ? stoch_cci_d : stoch_cci_k

trend_enter = if showArrowsEnter
    if crossunder(ma, OS)
        1
    else
        if crossover(ma, OB)
            -1
trend_exit = if showArrowsExit
    if crossunder(ma, OB)
        -1
    else
        if crossover(ma, OS)
            1

trend_center = if showArrowsCenter
    if crossunder(ma, 50)
        -1
    else
        if crossover(ma, 50)
            1
*/

fn calculate_direction(trade_generation_context: &TradeGenerationContext, candles: &Vec<Candle>) -> Direction {
  let cci_periods = trade_generation_context.cci_periods;
  let stoch_periods = trade_generation_context.stoch_periods;
  let stoch_smooth_k = 3;
  let stoch_smooth_d = 3;
  let overbought = 80.0;
  let oversold = 20.0;
  let ccis = pine_cci(candles, cci_periods);
  let stochs = pine_stoch(&ccis, stoch_periods);
  let stoch_cci_k = pine_sma(&stochs, stoch_smooth_k);
  let stoch_cci_d = pine_sma(&stoch_cci_k, stoch_smooth_d);
  if stoch_cci_d.len() > 0 {
    let ma = *stoch_cci_d.last().unwrap();
    if ma <= oversold {
      return Direction::Long;
    }
    if ma >= overbought {
      return Direction::Short;
    }
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
