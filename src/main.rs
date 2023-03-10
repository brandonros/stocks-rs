use std::{collections::HashMap, fs::File, time::Instant};

use chrono::{DateTime, Datelike, Duration, TimeZone, Weekday};
use chrono_tz::{Tz, US};
use csv::ReaderBuilder;
use memoize::memoize;
use ordered_float::OrderedFloat;
use rand::Rng;
use rand::rngs::ThreadRng;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use ta::{indicators::ExponentialMovingAverage, Next};

#[derive(PartialEq, Debug, Clone)]
enum Direction {
  Long,
  Short,
  Flat,
}

#[derive(PartialEq, Debug, Clone)]
enum MarketSessionType {
  None,
  Pre,
  Regular,
  Post,
}

enum Action {
  NoChange,
  Close,
  OpenNew,
  SwitchDirection,
}

struct Signal {
  pub grouping_key: i64,
  pub timestamp: i64,
  pub direction: Direction,
}

#[derive(Debug, Copy, Clone, Deserialize)]
struct Candle {
  pub start_timestamp: i64,
  pub end_timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

#[derive(PartialEq)]
enum TradeType {
  Open,
  Close,
}

struct Trade {
  pub grouping_key: i64,
  pub timestamp: i64,
  pub r#type: TradeType,
  pub direction: Direction,
}

#[derive(Debug)]
enum TradeExitType {
  Win,
  Loss,
}

#[derive(Debug)]
enum TradeExitReason {
  StopLoss,
  ProfitLimit,
  Close,
}

struct TradeBacktestResult {
  grouping_key: i64,
  open_timestamp: i64,
  exit_timestamp: i64,
  close_timestamp: i64,
  open_price: f64,
  close_price: f64,
  profit_limit_price: f64,
  stop_loss_price: f64,
  exit_reason: TradeExitReason,
  exit_candle: Candle,
  exit_price: f64,
  profit_loss: f64,
  profit_loss_percentage: f64,
  exit_type: TradeExitType,
}


#[derive(Debug, Clone)]
struct BacktestParameters {
  slippage_percentage: f64,
  profit_limit_percentage: f64,
  stop_loss_percentage: f64,
}

#[derive(Debug, Clone)]
struct SignalParameters {
  warmup_periods: usize,
  fast_periods: usize,
  slow_periods: usize,
}

fn build_decimal_range(min: Decimal, max: Decimal, step: Decimal) -> Vec<Decimal> {
  let mut pointer = min;
  let mut results = vec![];
  while pointer <= max {
    results.push(pointer);
    pointer += step;
  }
  return results;
}

fn build_usize_range(min: usize, max: usize, step: usize) -> Vec<usize> {
  let mut pointer = min;
  let mut results = vec![];
  while pointer <= max {
    results.push(pointer);
    pointer += step;
  }
  return results;
}

fn read_records_from_csv<T>(filename: &str) -> Vec<T>
where
  T: for<'de> Deserialize<'de>,
{
  let mut candles = vec![];
  let file = File::open(filename).unwrap();
  let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(file);
  for record in csv_reader.deserialize() {
    let candle: T = record.unwrap();
    candles.push(candle);
  }
  return candles;
}

#[memoize]
fn datetime_from_timestamp(timestamp: i64) -> DateTime<Tz> {
  let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
  return US::Eastern.from_utc_datetime(&naive);
}

#[memoize]
fn get_regular_market_session_start_and_end(timestamp: i64) -> (DateTime<Tz>, DateTime<Tz>) {
  let eastern_now = datetime_from_timestamp(timestamp);
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let start = US::Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let end = US::Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (start, end);
}

#[memoize]
fn determine_session_type(timestamp: i64) -> MarketSessionType {
  let eastern_now = datetime_from_timestamp(timestamp);
  // short circuit on weekends
  let weekday = eastern_now.weekday();
  let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
  if is_weekend {
    return MarketSessionType::None;
  }
  // short circuit on holidays
  let holidays_2022 = vec![
    "2022-01-17 00:00:00", // martin luther king jr day
    "2022-02-21 00:00:00", // preisdent's day
    "2022-04-15 00:00:00", // good friday
    "2022-05-30 00:00:00", // memorial day
    "2022-06-20 00:00:00", // juneteenth
    "2022-07-04 00:00:00", // independence day
    "2022-09-05 00:00:00", // labor day
    "2022-11-24 00:00:00", // day before thanksgiving
    "2022-11-25 00:00:00", // day after thanksgiving (closes at 1pm)?
    "2022-12-26 00:00:00", // day after christmas
  ];
  let holidays_2023 = vec![
    "2023-01-02 00:00:00", // new year's day
    "2023-01-16 00:00:00", // martin luther king jr day
    "2023-02-20 00:00:00", // preisdent's day
    "2023-04-07 00:00:00", // good friday
    "2023-05-29 00:00:00", // memorial day
    "2023-06-19 00:00:00", // juneteenth
    "2023-07-04 00:00:00", // independence day
    "2023-09-04 00:00:00", // labor day
    "2023-11-23 00:00:00", // thanksgiving day
    "2023-11-24 00:00:00", // day after thanksgiving (closes at 1pm)?
    "2023-12-25 00:00:00", // christmas
  ];
  let formatted_eastern_now = eastern_now.format("%Y-%m-%d 00:00:00").to_string();
  let is_2022_holiday = holidays_2022.iter().any(|&holiday| holiday == formatted_eastern_now);
  let is_2023_holiday = holidays_2023.iter().any(|&holiday| holiday == formatted_eastern_now);
  let is_holiday = is_2022_holiday || is_2023_holiday;
  if is_holiday {
    return MarketSessionType::None;
  }
  // check pre/regular/post
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  // premarket: 4am -> 9:29:59am
  let pre_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 4, 0, 0).unwrap();
  let pre_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 9, 29, 59).unwrap();
  let seconds_before_pre_market = eastern_now.signed_duration_since(pre_market_start).num_seconds();
  let seconds_after_pre_market = eastern_now.signed_duration_since(pre_market_end).num_seconds();
  let is_before_pre_market = seconds_before_pre_market < 0;
  let is_after_pre_market = seconds_after_pre_market >= 0;
  let is_during_pre_market = is_before_pre_market == false && is_after_pre_market == false;
  // regular: 9:30am -> 3:59:59pm
  let regular_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap();
  let regular_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap();
  let seconds_before_regular_market = eastern_now.signed_duration_since(regular_market_start).num_seconds();
  let seconds_after_regular_market = eastern_now.signed_duration_since(regular_market_end).num_seconds();
  let is_before_regular_market = seconds_before_regular_market < 0;
  let is_after_regular_market = seconds_after_regular_market >= 0;
  let is_during_regular_market = is_before_regular_market == false && is_after_regular_market == false;
  // aftermarket: 4:00pm -> 7:59:59pm
  let after_market_start = US::Eastern.with_ymd_and_hms(year, month, day, 16, 0, 0).unwrap();
  let after_market_end = US::Eastern.with_ymd_and_hms(year, month, day, 19, 59, 59).unwrap();
  let seconds_before_after_market = eastern_now.signed_duration_since(after_market_start).num_seconds();
  let seconds_after_after_market = eastern_now.signed_duration_since(after_market_end).num_seconds();
  let is_before_after_market = seconds_before_after_market < 0;
  let is_after_after_market = seconds_after_after_market >= 0;
  let is_during_after_market = is_before_after_market == false && is_after_after_market == false;
  if is_during_pre_market {
    return MarketSessionType::Pre;
  } else if is_during_regular_market {
    return MarketSessionType::Regular;
  } else if is_during_after_market {
    return MarketSessionType::Post;
  } else {
    return MarketSessionType::None;
  }
}

fn calculate_open_price(candle: &Candle, direction: &Direction, slippage_percentage: f64) -> f64 {
  if *direction == Direction::Long {
    return candle.open * (1.0 + slippage_percentage);
  } else {
    return candle.open * (1.0 - slippage_percentage);
  }
}

fn calculate_close_price(candle: &Candle, direction: &Direction, slippage_percentage: f64) -> f64 {
  if *direction == Direction::Long {
    return candle.open * (1.0 - slippage_percentage);
  } else {
    return candle.open * (1.0 + slippage_percentage);
  }
}

fn calculate_profit_limit_price(direction: &Direction, open_price: f64, profit_limit_percentage: f64) -> f64 {
  if *direction == Direction::Long {
    return open_price * (1.0 + profit_limit_percentage);
  } else {
    return open_price * (1.0 - profit_limit_percentage);
  }
}

fn calculate_stop_loss_price(direction: &Direction, open_price: f64, stop_loss_percentage: f64) -> f64 {
  if *direction == Direction::Long {
    return open_price * (1.0 - stop_loss_percentage.abs());
  } else {
    return open_price * (1.0 + stop_loss_percentage.abs());
  }
}

fn calculate_profit_loss(direction: &Direction, open_price: f64, exit_price: f64) -> f64 {
  if *direction == Direction::Long {
    return exit_price - open_price;
  } else {
    return open_price - exit_price;
  }
}

fn backtest_trade(
  trade_open: &Trade,
  trade_close: &Trade,
  candles_map: &HashMap<i64, &Candle>,
  backtest_parameters: &BacktestParameters,
  candle_size_seconds: i64,
) -> TradeBacktestResult {
  let slippage_percentage = backtest_parameters.slippage_percentage;
  let profit_limit_percentage = backtest_parameters.profit_limit_percentage;
  let stop_loss_percentage = backtest_parameters.stop_loss_percentage;
  // get candles
  let open_candle = candles_map.get(&trade_open.timestamp).unwrap();
  let close_candle = candles_map.get(&trade_close.timestamp).unwrap();
  // estimate open/close fill prices
  let open_price = calculate_open_price(&open_candle, &trade_open.direction, slippage_percentage);
  let close_price = calculate_close_price(&close_candle, &trade_open.direction, slippage_percentage);
  // estimate profit limit/stop loss prices
  let profit_limit_price = calculate_profit_limit_price(&trade_open.direction, open_price, profit_limit_percentage);
  let stop_loss_price = calculate_stop_loss_price(&trade_open.direction, open_price, stop_loss_percentage);
  // determine trade exit
  let determine_trade_exit = || {
    let mut pointer = trade_open.timestamp;
    while pointer < trade_close.timestamp { // do not include trade_close candle on purpose as to not introduce lookahead bias
      let candle = candles_map.get(&pointer).unwrap();
      // check for stop loss
      if trade_open.direction == Direction::Long {
        let worst_case_scenario = candle.low;
        if worst_case_scenario <= stop_loss_price {
          return (TradeExitReason::StopLoss, stop_loss_price, candle);
        }
      } else {
        let worst_case_scenario = candle.high;
        if worst_case_scenario >= stop_loss_price {
          return (TradeExitReason::StopLoss, stop_loss_price, candle);
        }
      }
      // check for profit limit
      if trade_open.direction == Direction::Long {
        let best_case_scenario = candle.high;
        if best_case_scenario >= profit_limit_price {
          return (TradeExitReason::ProfitLimit, profit_limit_price, candle);
        }
      } else {
        let best_case_scenario = candle.low;
        if best_case_scenario <= profit_limit_price {
          return (TradeExitReason::ProfitLimit, profit_limit_price, candle);
        }
      }
      // progress pointer through time
      pointer += candle_size_seconds;
    }
    // asume we close right at the open of the next candle due to direction change
    return (TradeExitReason::Close, close_price, close_candle);
  };
  let (exit_reason, exit_price, exit_candle) = determine_trade_exit();
  let profit_loss = calculate_profit_loss(&trade_open.direction, open_price, exit_price);
  let profit_loss_percentage = profit_loss / open_price;
  let exit_type = if profit_loss > 0.0 { TradeExitType::Win } else { TradeExitType::Loss };
  return TradeBacktestResult {
    grouping_key: trade_open.grouping_key,
    open_timestamp: trade_open.timestamp,
    exit_timestamp: exit_candle.start_timestamp,
    close_timestamp: trade_close.timestamp,
    open_price,
    close_price,
    profit_limit_price,
    stop_loss_price,
    exit_reason,
    exit_candle: *exit_candle.clone(),
    exit_price,
    profit_loss,
    profit_loss_percentage,
    exit_type,
  };
}

fn build_signals(candles: &Vec<Candle>, candles_map: &HashMap<i64, &Candle>, signal_parameters: &SignalParameters, candle_size_seconds: i64, rng: &mut ThreadRng) -> Vec<Signal> {
  let warmup_periods = signal_parameters.warmup_periods;
  let fast_periods = signal_parameters.fast_periods;
  let slow_periods = signal_parameters.slow_periods;
  // build indicators
  let mut fast = ExponentialMovingAverage::new(fast_periods).unwrap();
  let mut slow = ExponentialMovingAverage::new(slow_periods).unwrap();
  let mut last_fast = 0.0;
  let mut last_slow = 0.0;
  let mut num_periods = 0;
  // traverse time
  let parsed_start = datetime_from_timestamp(candles[0].start_timestamp);
  let parsed_end = datetime_from_timestamp(candles[candles.len() - 1].end_timestamp);
  let mut pointer = parsed_start;
  let mut signals = vec![];
  while pointer <= parsed_end {
    let current_session_type = determine_session_type(pointer.timestamp());
    // skip when market is not open
    if current_session_type == MarketSessionType::None {
      pointer = pointer + Duration::seconds(candle_size_seconds);
      continue;
    }
    // TODO: prediction/estimation so that we aren't always late to trades?
    // get previous fully closed candle (alway look back 1 candle to prevent lookahead bias)
    let massaged_timestamp = pointer.timestamp() - candle_size_seconds;
    let previous_candle = candles_map.get(&massaged_timestamp);
    if previous_candle.is_none() {
      if current_session_type == MarketSessionType::Regular {
        panic!("no candle for {pointer} {massaged_timestamp}?");
      }
      // skip missing pre/post market candles
      pointer = pointer + Duration::seconds(candle_size_seconds);
      continue;
    }
    let previous_candle = previous_candle.unwrap();
    // feed to indicators
    last_fast = fast.next(previous_candle.close);
    last_slow = slow.next(previous_candle.close);
    // get only open price from current candle to prevent lookahead bias
    let current_candle = candles_map.get(&pointer.timestamp());
    if current_candle.is_none() {
      if current_session_type == MarketSessionType::Regular {
        panic!("no candle for {pointer} {massaged_timestamp}?");
      }
      // skip missing pre/post market candles
      pointer = pointer + Duration::seconds(candle_size_seconds);
      continue;
    }
    let current_candle = current_candle.unwrap();
    // feed to indicators (pretend we can accurately predict close)
    let accuracy = 0.45;
    let predicted_close_correctly = rng.gen_bool(accuracy) == true;
    if predicted_close_correctly == true {
      last_fast = fast.next(current_candle.close);
      last_slow = slow.next(current_candle.close);
    } else {
      last_fast = fast.next(current_candle.open);
      last_slow = slow.next(current_candle.open);
    }
    // calculate indicator direction
    let indicator_direction = if last_fast > last_slow {
      Direction::Long
    } else {
      Direction::Short
    };
    num_periods += 1;
    // calculate warmup
    let is_warmed_up = num_periods >= warmup_periods;
    // calculate direction
    let is_pre_market = current_session_type == MarketSessionType::Pre;
    let is_post_market = current_session_type == MarketSessionType::Post;
    let (regular_session_start, regular_session_end) = get_regular_market_session_start_and_end(pointer.timestamp());
    let distance_to_regular_session_end = regular_session_end.timestamp() - pointer.timestamp();
    let is_last_candle_of_regular_session = current_session_type == MarketSessionType::Regular && distance_to_regular_session_end <= (candle_size_seconds - 1);
    let should_be_flat = is_warmed_up == false || is_pre_market || is_post_market || is_last_candle_of_regular_session;
    let direction = if should_be_flat {
      Direction::Flat
    } else {
      indicator_direction
    };
    // push
    signals.push(Signal {
      grouping_key: regular_session_start.timestamp(),
      timestamp: pointer.timestamp(), // use pointer timestamp, not use candle timestamp to prevent lookahead bias
      direction,
    });
    // increment
    pointer = pointer + Duration::seconds(candle_size_seconds);
  }
  return signals;
}

fn build_trades(signals: &Vec<Signal>) -> Vec<Trade> {
  let mut trades = vec![];
  let mut last_direction = Direction::Flat;
  for signal in signals {
    let signal_direction = &signal.direction;
    let action = match (&last_direction, signal_direction) {
      // stay in (no change)
      (Direction::Short, Direction::Short) => Action::NoChange,
      (Direction::Long, Direction::Long) => Action::NoChange,
      (Direction::Flat, Direction::Flat) => Action::NoChange,
      // get out (close)
      (Direction::Long, Direction::Flat) => Action::Close,
      (Direction::Short, Direction::Flat) => Action::Close,
      // open new
      (Direction::Flat, Direction::Long) => Action::OpenNew,
      (Direction::Flat, Direction::Short) => Action::OpenNew,
      // switch direction
      (Direction::Short, Direction::Long) => Action::SwitchDirection,
      (Direction::Long, Direction::Short) => Action::SwitchDirection,
    };
    match action {
      Action::OpenNew => {
        trades.push(Trade {
          grouping_key: signal.grouping_key,
          timestamp: signal.timestamp,
          r#type: TradeType::Open,
          direction: signal.direction.clone(),
        });
      }
      Action::NoChange => {}
      Action::Close => {
        trades.push(Trade {
          grouping_key: signal.grouping_key,
          timestamp: signal.timestamp,
          r#type: TradeType::Close,
          direction: last_direction,
        });
      }
      Action::SwitchDirection => {
        trades.push(Trade {
          grouping_key: signal.grouping_key,
          timestamp: signal.timestamp,
          r#type: TradeType::Close,
          direction: last_direction,
        });
        trades.push(Trade {
          grouping_key: signal.grouping_key,
          timestamp: signal.timestamp,
          r#type: TradeType::Open,
          direction: signal.direction.clone(),
        });
      }
    }
    last_direction = signal.direction.clone();
  }
  return trades;
}

fn build_backtest_parameter_combinations() -> Vec<BacktestParameters> {
  let mut backtest_parameter_combinations = vec![];
  let min = dec!(0.002);
  let max = dec!(0.01);
  let step = dec!(0.0005);
  let profit_limit_percentages = build_decimal_range(min, max, step);
  let min = dec!(-0.01);
  let max = dec!(-0.002);
  let step = dec!(0.0005);
  let stop_loss_percentages = build_decimal_range(min, max, step);
  for profit_limit_percentage in &profit_limit_percentages {
    for stop_loss_percentage in &stop_loss_percentages {
      let backtest_parameters = BacktestParameters {
        slippage_percentage: 0.000125,
        profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
        stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
      };
      backtest_parameter_combinations.push(backtest_parameters);
    }
  }
  return backtest_parameter_combinations;
}

fn build_signal_parameter_combinations() -> Vec<SignalParameters> {
  let mut signal_parameter_combinations = vec![];
  let min = 10;
  let max = 50;
  let step = 5;
  let fast_periods = build_usize_range(min, max, step);
  let min = 20;
  let max = 100;
  let step = 10;
  let slow_periods = build_usize_range(min, max, step);
  for slow_periods in &slow_periods {
    for fast_periods in &fast_periods {
      // skip where fast is greater than or equal to slow?
      if fast_periods >= slow_periods {
        continue;
      }
      let backtest_context = SignalParameters {
        warmup_periods: 1,
        fast_periods: *fast_periods,
        slow_periods: *slow_periods,
      };
      signal_parameter_combinations.push(backtest_context);
    }
  }
  return signal_parameter_combinations;
}

fn main() {
  // rng
  let mut rng = rand::thread_rng();
  // load candles
  let resolution = 1;
  let candles_filename = format!("./output/candles-{resolution}.csv");
  let candle_size_seconds = resolution * 60;
  let candles = read_records_from_csv::<Candle>(&candles_filename);
  let mut candles_map = HashMap::new();
  for candle in &candles {
    candles_map.insert(candle.start_timestamp, candle);
  }
  // build all possible signal/trade combinations
  let mut total_performance_map = std::collections::BTreeMap::new();
  let backtest_parameter_combinations = build_backtest_parameter_combinations();
  let signal_parameter_combinations = build_signal_parameter_combinations();
  for signal_parameters in &signal_parameter_combinations {
    // build signals
    let signals = build_signals(&candles, &candles_map, &signal_parameters, candle_size_seconds, &mut rng);
    // build trades from signals
    let trades = build_trades(&signals);
    let trades_slice: &[Trade] = &trades;
    let chunk_size = 2; // open + close
    let chunked_trades: Vec<&[Trade]> = trades_slice.chunks(chunk_size).collect();
    for chunk in chunked_trades {
      // get open + close from chunk
      let trade_open = &chunk[0];
      let trade_close = &chunk[1];
      assert!(trade_open.r#type == TradeType::Open);
      assert!(trade_close.r#type == TradeType::Close);
      assert!(trade_open.direction == trade_close.direction);
      assert!(trade_open.timestamp != trade_close.timestamp);
      // loop backtest parameter combinations
      for backtest_parameters in &backtest_parameter_combinations {
        // backtest trade
        let backtest_result = backtest_trade(trade_open, trade_close, &candles_map, backtest_parameters, candle_size_seconds);
        // record performance
        let profit_loss_percentage = backtest_result.profit_loss_percentage;
        let profit_limit_percentage = backtest_parameters.profit_limit_percentage;
        let stop_loss_percentage = backtest_parameters.stop_loss_percentage;
        let fast_periods = signal_parameters.fast_periods;
        let slow_periods = signal_parameters.slow_periods;
        let signal_key = (fast_periods, slow_periods);
        let size_key = (OrderedFloat(profit_limit_percentage), OrderedFloat(stop_loss_percentage));
        let total_key = (signal_key, size_key);
        if total_performance_map.contains_key(&total_key) == false {
          total_performance_map.insert(total_key, 0.0);
        }
        let total_key_performance = total_performance_map.get(&total_key).unwrap();
        total_performance_map.insert(total_key, total_key_performance + profit_loss_percentage);
      }
    }
  }
  // print results
  println!("fast_periods,slow_periods,profit_limit_percentage,stop_loss_percentage,profit_loss_percentage");
  for (key, profit_loss_percentage) in total_performance_map.iter() {
    let fast_periods = key.0.0;
    let slow_periods = key.0.1;
    let profit_limit_percentage = key.1.0;
    let stop_loss_percentage = key.1.1;
    println!("{fast_periods},{slow_periods},{profit_limit_percentage},{stop_loss_percentage},{profit_loss_percentage}");
  }
}