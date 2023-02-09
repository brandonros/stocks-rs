use std::{sync::Arc, collections::HashMap};

use ta::Next;

use crate::{structs::*, market_session};

fn calculate_trades_from_direction_snapshots(direction_snapshots: &Vec<Arc<DirectionSnapshot>>) -> Vec<Trade> {
  let mut buckets: Vec<Vec<Arc<DirectionSnapshot>>> = Vec::new();
  let mut bucket: Vec<Arc<DirectionSnapshot>> = Vec::new();
  let mut current_direction = &direction_snapshots[0].direction;
  for direction_snapshot in direction_snapshots {
    if direction_snapshot.direction != *current_direction {
      buckets.push(bucket);
      bucket = Vec::new();
      current_direction = &direction_snapshot.direction;
    }
    bucket.push(direction_snapshot.clone());
  }
  if bucket.is_empty() == false {
    buckets.push(bucket);
  }
  return buckets
    .into_iter()
    .map(|bucket| {
      return Trade {
        start_timestamp: bucket[0].timestamp,
        end_timestamp: bucket[bucket.len() - 1].timestamp,
        direction: bucket[0].direction,
      };
    })
    .collect();
}

fn get_vwap(candles: &Vec<Arc<Candle>>, std_dev_multiplier: f64) -> VwapContext {
  // build indicators
  let mut indicator = ta::indicators::VolumeWeightedAveragePrice::new();
  // loop candles
  let mut last_vwap_upper_band = 0.0;
  let mut last_vwap_lower_band = 0.0;
  let mut last_vwap = 0.0;
  for candle in candles {
    let open = candle.open;
    let high = candle.high;
    let low = candle.low;
    let close = candle.close;
    let volume = candle.volume as f64;
    let data_item = ta::DataItem::builder()
      .high(high)
      .low(low)
      .close(close)
      .open(open)
      .volume(volume)
      .build()
      .unwrap();
    last_vwap = indicator.next(&data_item);
    last_vwap_upper_band = indicator.std_dev(std_dev_multiplier, ta::indicators::VolumeWeightedAveragePriceBands::Up);
    last_vwap_lower_band = indicator.std_dev(std_dev_multiplier, ta::indicators::VolumeWeightedAveragePriceBands::Down);
  }
  return VwapContext {
    vwap: last_vwap,
    upper_band: last_vwap_upper_band,
    lower_band: last_vwap_lower_band,
  };
}

fn get_hlc3_sma(candles: &Vec<Arc<Candle>>, periods: usize) -> f64 {
  // build indicators
  let mut indicator = ta::indicators::SimpleMovingAverage::new(periods).unwrap();
  // loop candles
  let mut last_sma = 0.0;
  for candle in candles {
    let high = candle.high;
    let low = candle.low;
    let close = candle.close;
    let hlc3 = (high + low + close) / 3.0;
    last_sma = indicator.next(hlc3);
  }
  return last_sma;
}

fn generate_direction_snapshots(
  trade_generation_context: &TradeGenerationContext,
  date: &str,
  date_candles: &Vec<Arc<Candle>>,
  strategy_name: &str,
) -> Vec<Arc<DirectionSnapshot>> {
  assert!(strategy_name == "vwap_hlc3_divergence"); // TODO: more strategies?
  let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
  let mut pointer = regular_market_start;
  let mut direction_snapshots: Vec<Arc<DirectionSnapshot>> = vec![];
  // iterate over every minute of the trading day, making sure we do not include the end of the most recent candle because it would not be known in a live situation
  while pointer <= regular_market_end {
    let reduced_candles: Vec<Arc<Candle>> = date_candles
      .iter()
      .filter(|candle| return candle.timestamp < pointer.timestamp())
      .cloned()
      .collect();
    // allow warmup
    if reduced_candles.len() < trade_generation_context.warmup_periods {
      pointer += chrono::Duration::minutes(1);
      continue;
    }
    // calculate vwap
    let vwap_context = get_vwap(&reduced_candles, trade_generation_context.vwap_std_dev_multiplier);
    // calculate hlc3 sma
    let hlc3_sma = get_hlc3_sma(&reduced_candles, trade_generation_context.sma_periods);
    // get divergence percentage
    let divergence_percentage = (hlc3_sma - vwap_context.vwap) / vwap_context.vwap;
    /*log::info!(
      "vwap = {:.2} hlc3_sma = {:2} divergence_percentage = {:4}",
      vwap_context.vwap,
      hlc3_sma,
      divergence_percentage
    );*/
    let direction = if divergence_percentage > trade_generation_context.divergence_threshold {
      Direction::Long
    } else {
      Direction::Short
    };
    direction_snapshots.push(Arc::new(DirectionSnapshot {
      timestamp: pointer.timestamp(),
      direction,
    }));
    pointer += chrono::Duration::minutes(1);
  }
  return direction_snapshots;
}

pub fn generate_dates_trades_map(dates: &Vec<String>, trade_generation_context: &TradeGenerationContext, strategy_name: &str, candles_date_map: &HashMap<String, Vec<Arc<Candle>>>) -> HashMap<String, Vec<Trade>> {
  let mut dates_trades_map = HashMap::new();
  for date in dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let direction_snapshots = generate_direction_snapshots(&trade_generation_context, date, date_candles, &strategy_name);
    if direction_snapshots.is_empty() {
      log::warn!("date = {} direction_snapshots.is_empty()", date);
      dates_trades_map.insert(date.clone(), vec![]);
      continue;
    }
    let date_trades = calculate_trades_from_direction_snapshots(&direction_snapshots);
    log::info!("date = {} num_trades = {}", date, date_trades.len());
    dates_trades_map.insert(date.clone(), date_trades);
  }
  return dates_trades_map;
}
