use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::DateTime;
use chrono::Timelike;
use chrono_tz::Tz;
use common::database;
use common::database::Database;
use common::dates;
use common::market_session;
use common::math;
use common::structs::*;
use common::utilities;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use ta::Next;

#[derive(Serialize, Debug, Clone, PartialEq)]
enum EventDescription {
  SwitchDirection,
  FireLong,
  FireShort,
}

#[derive(Debug, Serialize, Clone)]
struct BacktestContext {
  pub atr_periods: usize,
  pub supertrend_periods: usize,
  pub supertrend_multiplier: f64,
  pub profit_limit_percentage: f64,
  pub stop_loss_percentage: f64,
  pub entry_mode: EntryMode,
}

#[derive(Debug, Deserialize, Clone)]
struct Candle {
  pub timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: usize,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
enum EventType {
  Open,
  Close,
}

#[derive(Serialize, Debug, Clone)]
struct Event {
  pub timestamp: i64,
  pub description: EventDescription,
  pub r#type: EventType,
  pub price: f64,
}

#[derive(Serialize, Clone)]
struct DirectionWindow {
  pub start_event: Event,
  pub end_event: Event,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
enum EntryMode {
  Single,
  Stacked,
}

#[derive(Serialize, Debug)]
enum Outcome {
  StopLoss,
  ProfitLimit,
  DirectionChange,
}

#[derive(Serialize, Debug)]
struct TradeResult {
  pub direction: Direction,
  pub start_timestamp: i64,
  pub exit_timestamp: i64,
  pub outcome: Outcome,
  pub open_atr: f64,
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

fn get_candle_snapshots_from_database(
  connection: &Database,
  symbol: &str,
  resolution: &str,
  regular_market_start_timestamp: i64,
  regular_market_end_timestamp: i64,
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
    where timestamp >= {regular_market_start_timestamp} and timestamp <= {regular_market_end_timestamp}
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
    "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

fn get_direction(candles: &Vec<&Candle>, supertrend_periods: usize, supertrend_multiplier: f64) -> Direction {
  // build indicators
  let mut supertrend_indicator = ta::indicators::Supertrend::new(supertrend_periods, supertrend_multiplier);
  // loop candles
  let mut last_direction = Direction::Flat;
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
    // supertrend
    let (_supertrend_upper_band, _supertrend_lower_band, supertrend_direction) = supertrend_indicator.next(&data_item);
    last_direction = if supertrend_direction == -1 { Direction::Short } else { Direction::Long };
  }
  return last_direction;
}

fn get_atr(candles: &Vec<&Candle>, atr_periods: usize) -> f64 {
  // build indicators
  let mut indicator = ta::indicators::AverageTrueRange::new(atr_periods).unwrap();
  // loop candles
  let mut last_atr = 0.0;
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
    last_atr = indicator.next(&data_item);
  }
  return last_atr;
}

fn calculate_slippage_percentage(_backtest_context: &BacktestContext, current_atr: f64) -> f64 {
  let is_low_atr = current_atr <= 0.25;
  let is_high_atr = current_atr >= 0.45;
  let is_medium_atr = is_low_atr == false && is_high_atr == false;
  // TODO: does this scaling by ATR help or hurt and are the values realistic?
  if is_low_atr  {
    return 0.00015;
  } else if is_medium_atr {
    return 0.00020;
  } else if is_high_atr {
    return 0.00025; 
  } else {
    unreachable!()
  }
}

fn calculate_profit_limit_percentage(backtest_context: &BacktestContext, current_atr: f64) -> f64 {
  // TODO: scale based on ATR?
  return backtest_context.profit_limit_percentage;
}

fn calculate_stop_loss_percentage(backtest_context: &BacktestContext, current_atr: f64) -> f64 {
  // TODO: scale based on ATR?
  return backtest_context.stop_loss_percentage;
}

fn calculate_trade_result(
  backtest_context: &BacktestContext,
  trade_candles: &Vec<Candle>,
  trade_direction: &Direction,
  start_timestamp: i64,
  open_price: f64,
  open_atr: f64
) -> TradeResult {
  // do not include the last candle because it's only included for direction change purposes (getting out right at open)
  for i in 0..trade_candles.len() - 1 {
    let trade_candle = &trade_candles[i];
    // worst case scenario first based on direction for stop loss
    let hypothetical_exit_price = if *trade_direction == Direction::Long {
      trade_candle.low
    } else {
      trade_candle.high
    };
    // dynamic trade sizing based on atr
    let slippage_percentage = calculate_slippage_percentage(backtest_context, open_atr);
    let stop_loss_percentage = calculate_stop_loss_percentage(backtest_context, open_atr);
    let profit_limit_percentage = calculate_profit_limit_percentage(backtest_context, open_atr);
    // exit price
    let hypothetical_exit_price = math::calculate_close_price_with_slippage(trade_direction, hypothetical_exit_price, slippage_percentage);
    let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, hypothetical_exit_price);
    let stop_loss_hit = profit_loss_percentage <= stop_loss_percentage;
    if stop_loss_hit {
      // force exit price to be capped to stop_loss_price at worse
      let stop_loss_price = math::calculate_stop_loss_price(trade_direction, open_price, stop_loss_percentage);
      let exit_price = stop_loss_price;
      let profit_loss = math::calculate_profit_loss(trade_direction, open_price, stop_loss_price);
      let profit_loss_percentage = stop_loss_percentage;
      return TradeResult {
        open_price,
        open_atr,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        outcome: Outcome::StopLoss,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
    // best case scenario next based on direction for profit limit
    let hypothetical_exit_price = if *trade_direction == Direction::Long {
      trade_candle.high
    } else {
      trade_candle.low
    };
    let hypothetical_exit_price = math::calculate_close_price_with_slippage(&trade_direction, hypothetical_exit_price, slippage_percentage);
    let profit_loss_percentage = math::calculate_profit_loss_percentage(&trade_direction, open_price, hypothetical_exit_price);
    if profit_loss_percentage >= profit_limit_percentage {
      // force exit price to be capped to profit_limit_price at best
      let profit_limit_price = math::calculate_profit_limit_price(&trade_direction, open_price, profit_limit_percentage);
      let exit_price = profit_limit_price;
      let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, profit_limit_price);
      let profit_loss_percentage = profit_limit_percentage;
      return TradeResult {
        open_price,
        open_atr,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        outcome: Outcome::ProfitLimit,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
  }
  // dynamic trade sizing based on atr
  let slippage_percentage = calculate_slippage_percentage(backtest_context, open_atr);
  let stop_loss_percentage = calculate_stop_loss_percentage(backtest_context, open_atr);
  let profit_limit_percentage = calculate_profit_limit_percentage(backtest_context, open_atr);
  // exit on last candle
  let trade_end_candle = &trade_candles[trade_candles.len() - 1];
  let exit_price = trade_end_candle.open;
  let exit_price = math::calculate_close_price_with_slippage(&trade_direction, exit_price, slippage_percentage);
  let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, exit_price);
  let profit_loss_percentage = math::calculate_profit_loss_percentage(&trade_direction, open_price, exit_price);
  if profit_loss_percentage < stop_loss_percentage {
    log::warn!("{} < stop_loss_percentage", profit_loss_percentage);
  }
  if profit_loss_percentage > profit_limit_percentage {
    log::warn!("{} > profit_limit_percentage", profit_loss_percentage);
  }
  return TradeResult {
    open_price,
    open_atr,
    direction: trade_direction.clone(),
    start_timestamp,
    exit_timestamp: trade_end_candle.timestamp,
    outcome: Outcome::DirectionChange,
    exit_price,
    profit_loss,
    profit_loss_percentage,
  };
}

fn calculate_directions(backtest_context: &BacktestContext, start: DateTime<Tz>, end: DateTime<Tz>, candles: &Vec<Candle>) -> HashMap<i64, Direction> {
  let mut pointer = start;
  let mut output = HashMap::new();
  while pointer <= end {
    let reduced_candles: Vec<&Candle> = candles.iter().filter(|candle| return candle.timestamp < pointer.timestamp()).collect();
    // allow warmup
    if reduced_candles.len() < backtest_context.supertrend_periods {
      pointer += chrono::Duration::minutes(1);
      continue;
    }
    let current_direction = get_direction(&reduced_candles, backtest_context.supertrend_periods, backtest_context.supertrend_multiplier);
    output.insert(pointer.timestamp(), current_direction);
    pointer += chrono::Duration::minutes(1);
  }
  return output;
}

fn calculate_atrs(backtest_context: &BacktestContext, start: DateTime<Tz>, end: DateTime<Tz>, candles: &Vec<Candle>) -> HashMap<i64, f64> {
  let mut pointer = start;
  let mut output = HashMap::new();
  while pointer <= end {
    let reduced_candles: Vec<&Candle> = candles.iter().filter(|candle| return candle.timestamp < pointer.timestamp()).collect();
    // allow warmup
    if reduced_candles.len() < backtest_context.supertrend_periods {
      pointer += chrono::Duration::minutes(1);
      continue;
    }
    let current_atr = get_atr(&reduced_candles, backtest_context.atr_periods);
    output.insert(pointer.timestamp(), current_atr);
    pointer += chrono::Duration::minutes(1);
  }
  return output;
}

fn calculate_events(backtest_context: &BacktestContext, start: DateTime<Tz>, end: DateTime<Tz>, candles: &Vec<Candle>, timestamps_candles_map: &HashMap<i64, &Candle>, timestamps_directions_map: &HashMap<i64, Direction>, timestamps_atrs_map: &HashMap<i64, f64>) -> Vec<Event> {
  let mut pointer = start;
  let mut last_direction = Direction::Flat;
  let mut last_trade_start = None;
  let mut events = vec![];
  while pointer <= end {
    let is_end_of_day = pointer.hour() == 15 && pointer.minute() == 59;
    let reduced_candles: Vec<&Candle> = candles.iter().filter(|candle| return candle.timestamp < pointer.timestamp()).collect();
    // allow warmup
    if reduced_candles.len() < backtest_context.supertrend_periods {
      pointer += chrono::Duration::minutes(1);
      continue;
    }
    // find current candle
    let current_candle = *timestamps_candles_map.get(&pointer.timestamp()).unwrap();
    let current_direction = *timestamps_directions_map.get(&pointer.timestamp()).unwrap();
    let current_atr = *timestamps_atrs_map.get(&pointer.timestamp()).unwrap();
    if current_direction != last_direction {
      // close any existing trades
      if last_trade_start.is_some() {
        let slippage_percentage = calculate_slippage_percentage(backtest_context, current_atr);
        let hypothetical_close_price = current_candle.open;
        let hypothetical_close_price_with_slippage =
          math::calculate_close_price_with_slippage(&last_direction, hypothetical_close_price, slippage_percentage);
        events.push(Event {
          timestamp: pointer.timestamp(),
          //formatted_timestamp: pointer.format("%Y-%m-%d %I:%M:%S %p").to_string(),
          description: EventDescription::SwitchDirection,
          r#type: EventType::Close,
          price: hypothetical_close_price_with_slippage,
        });
      }
      // only take new trade if it isn't end of day
      if is_end_of_day == false {
        let slippage_percentage = calculate_slippage_percentage(backtest_context, current_atr);
        let hypothetical_open_price = current_candle.open;
        let hypothetical_open_price_with_slippage =
          math::calculate_open_price_with_slippage(&current_direction, hypothetical_open_price, slippage_percentage);
        events.push(Event {
          timestamp: pointer.timestamp(),
          //formatted_timestamp: pointer.format("%Y-%m-%d %I:%M:%S %p").to_string(),
          description: if current_direction == Direction::Long {
            EventDescription::FireLong
          } else {
            EventDescription::FireShort
          },
          r#type: EventType::Open,
          price: hypothetical_open_price_with_slippage,
        });
        last_direction = current_direction;
        last_trade_start = Some(pointer.clone());
      }
    }
    // make sure we always end on a close
    if is_end_of_day {
      let last_event = &events[events.len() - 1];
      if last_event.r#type != EventType::Close {
        let slippage_percentage = calculate_slippage_percentage(backtest_context, current_atr);
        let hypothetical_close_price = current_candle.open;
        let hypothetical_close_price_with_slippage =
          math::calculate_close_price_with_slippage(&last_direction, hypothetical_close_price, slippage_percentage);
        events.push(Event {
          timestamp: pointer.timestamp(),
          //formatted_timestamp: pointer.format("%Y-%m-%d %I:%M:%S %p").to_string(),
          description: EventDescription::SwitchDirection,
          r#type: EventType::Close,
          price: hypothetical_close_price_with_slippage,
        });
      }
    }
    pointer += chrono::Duration::minutes(1);
  }
  // make sure we always end on a close
  if events.len() > 0 {
    assert!(events[events.len() - 1].r#type == EventType::Close);
  }
  return events;
}

fn debug_trade_result(trade_result: &TradeResult) {
  let trade_result_type = if trade_result.profit_loss > 0.0 {
    String::from("win")
  } else {
    String::from("loss")
  };
  let is_low_atr = trade_result.open_atr <= 0.25;
  let is_high_atr = trade_result.open_atr >= 0.45;
  let is_medium_atr = is_low_atr == false && is_high_atr == false;
  let trade_atr_type = if is_low_atr {
    String::from("low")
  } else if is_medium_atr {
    String::from("medium")
  } else if is_high_atr {
    String::from("high")
  } else {
    unreachable!()
  };
  let mut row = vec![];
  row.push(format!("{}", dates::format_timestamp(trade_result.start_timestamp)));
  row.push(format!("{:?}", trade_result.direction));
  row.push(format!("${:.2}", trade_result.open_price));
  row.push(format!("${:.2}", trade_result.open_atr));
  row.push(format!("{:?}", trade_result.outcome));
  row.push(format!("{}", dates::format_timestamp(trade_result.exit_timestamp)));
  row.push(format!("${:.2}", trade_result.exit_price));
  row.push(format!("${:.2}", trade_result.profit_loss));
  row.push(format!("{:.4}", trade_result.profit_loss_percentage));
  row.push(trade_result_type);
  row.push(trade_atr_type);
  log::info!("{}", row.join(","));
}

fn backtest_date(backtest_context: &BacktestContext, start: DateTime<Tz>, end: DateTime<Tz>, candles: &Vec<Candle>, timestamps_candles_map: &HashMap<i64, &Candle>, timestamps_directions_map: &HashMap<i64, Direction>, timestamps_atrs_map: &HashMap<i64, f64>) -> Vec<TradeResult> {
  let events = calculate_events(backtest_context, start, end, &candles, timestamps_candles_map, timestamps_directions_map, timestamps_atrs_map);
  let direction_windows: Vec<DirectionWindow> = events
    .chunks(2)
    .into_iter()
    .map(|chunk| {
      assert!(chunk[0].r#type == EventType::Open);
      assert!(chunk[1].r#type == EventType::Close);
      return DirectionWindow {
        start_event: chunk[0].clone(),
        end_event: chunk[1].clone(),
      };
    })
    /*.filter(|direction_window| {
      return direction_window.start_event.description == EventDescription::FireLong; // warning: only ever taking long trades, no short
    })*/
    .filter(|direction_window| {
      let open_atr = *timestamps_atrs_map.get(&direction_window.start_event.timestamp).unwrap();
      let is_low_atr = open_atr <= 0.25;
      let is_high_atr = open_atr >= 0.45;
      let is_medium_atr = is_low_atr == false && is_high_atr == false;
      return is_medium_atr || is_high_atr; // experiment with only taking medium/high ATR trades
    })
    .collect();
  let entry_mode = &backtest_context.entry_mode;
  let mut trade_results = vec![];
  for direction_window in &direction_windows {
    let direction_window_direction = if direction_window.start_event.description == EventDescription::FireLong {
      Direction::Long
    } else {
      Direction::Short
    };
    let direction_window_candles: Vec<Candle> = candles
      .iter()
      .cloned()
      .filter(|candle| {
        return candle.timestamp >= direction_window.start_event.timestamp && candle.timestamp <= direction_window.end_event.timestamp;
      })
      .collect();
    let mut i = 0;
    while i < direction_window_candles.len() - 1 {
      // do not include last candle because it's a direction change
      let open_candle = &direction_window_candles[i];
      let open_atr = *timestamps_atrs_map.get(&open_candle.timestamp).unwrap();
      let slippage_percentage = calculate_slippage_percentage(backtest_context, open_atr);
      let open_price = math::calculate_open_price_with_slippage(&direction_window_direction, open_candle.open, slippage_percentage);
      let trade_candles = &direction_window_candles[i..direction_window_candles.len()].to_vec();
      let trade_result = calculate_trade_result(backtest_context, &trade_candles, &direction_window_direction, open_candle.timestamp, open_price, open_atr);
      let trade_duration = (trade_result.exit_timestamp - trade_result.start_timestamp) / 60;
      trade_results.push(trade_result);
      // logic to prevent stacked vs single entry per direction window
      if *entry_mode == EntryMode::Single {
        break;
      }
      // skip at least one minute if we got stopped out or profit limited immediately on the same candle we opened on
      if trade_duration == 0 {
        i += 1;
      } else {
        i += trade_duration as usize;
      }
    }
  }
  return trade_results;
}

fn backtest_dates(backtest_context: &BacktestContext, dates: &Vec<String>, candles_map: &HashMap<String, Vec<Candle>>) -> (f64, Vec<TradeResult>) {
  let trade_results: Vec<TradeResult> = dates.iter().fold(vec![], |mut prev, date| {
    let date_candles = candles_map.get(date).unwrap();
    // skip dates with no candles?
    if date_candles.len() == 0 {
      return prev;
    }
    let timestamps_candles_map = date_candles.iter().fold(HashMap::new(), |mut acc, candle| {
      acc.insert(candle.timestamp, candle);
      acc
    });
    let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
    let timestamps_directions_map = calculate_directions(backtest_context, regular_market_start, regular_market_end, date_candles);
    let timestamps_atrs_map = calculate_atrs(backtest_context, regular_market_start, regular_market_end, date_candles);
    let date_results = backtest_date(backtest_context, regular_market_start, regular_market_end, date_candles, &timestamps_candles_map, &timestamps_directions_map, &timestamps_atrs_map);
    prev.extend(date_results);
    return prev;
  });
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  for i in 0..trade_results.len() {
    let trade_result = &trade_results[i];
    balance *= 1.0 + trade_result.profit_loss_percentage;
  }
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  return (compounded_profit_loss_percentage, trade_results);
}

fn build_combinations() -> Vec<BacktestContext> {
  let supertrend_periods: Vec<usize> = (5..30).step_by(1).collect();
  //let supertrend_multipliers = build_decimal_range(dec!(0.25), dec!(3.0), dec!(0.25));
  let supertrend_multipliers = utilities::build_decimal_range(dec!(2.00), dec!(3.0), dec!(0.25));
  let profit_limit_percentages = utilities::build_decimal_range(dec!(0.001), dec!(0.005), dec!(0.0005));
  let stop_loss_percentages = utilities::build_decimal_range(dec!(-0.005), dec!(-0.001), dec!(0.0005));
  let mut combinations = vec![];
  for profit_limit_percentage in &profit_limit_percentages {
    for stop_loss_percentage in &stop_loss_percentages {
      for supertrend_periods in &supertrend_periods {
        for supertrend_multiplier in &supertrend_multipliers {
          combinations.push(BacktestContext {
            supertrend_periods: *supertrend_periods,
            supertrend_multiplier: supertrend_multiplier.to_f64().unwrap(),
            profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
            stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
            atr_periods: *supertrend_periods, // TODO: test different values
            entry_mode: EntryMode::Single,
          });
        }
      }
    }
  }
  return combinations;
  /*return vec![BacktestContext {
    supertrend_periods: 10,
    supertrend_multiplier: 3.00,
    profit_limit_percentage: 0.002,
    stop_loss_percentage: -0.01,
    entry_mode: EntryMode::Single,
    atr_periods: 10, // TODO: test different values
  }];*/
}

fn main() {
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // config
  let args: Vec<String> = std::env::args().collect();
  let provider_name = args.get(1).unwrap();
  let _strategy_name = args.get(2).unwrap();
  let symbol = args.get(3).unwrap();
  let resolution = args.get(4).unwrap();
  let dates_start = format!("{} 00:00:00", args.get(5).unwrap());
  let dates_end = format!("{} 00:00:00", args.get(6).unwrap());
  let dates = common::dates::build_list_of_dates(&dates_start, &dates_end);
  // open database + init database tables
  let database_filename = format!("./database-{}.db", provider_name);
  let connection = database::Database::new(&database_filename);
  connection.migrate("./schema/");
  // build candles cache map
  let mut candles_map = HashMap::new();
  for date in &dates {
    let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let regular_market_start_timestamp = regular_market_start.timestamp();
    let regular_market_end_timestamp = regular_market_end.timestamp();
    // get candles from database
    let candle_snapshots = get_candle_snapshots_from_database(&connection, symbol, resolution, regular_market_start_timestamp, regular_market_end_timestamp);
    let candles: Vec<Candle> = candle_snapshots
      .iter()
      .map(|candle_snapshot| {
        return Candle {
          timestamp: candle_snapshot.timestamp,
          open: candle_snapshot.open,
          high: candle_snapshot.high,
          low: candle_snapshot.low,
          close: candle_snapshot.close,
          volume: candle_snapshot.volume as usize,
        };
      })
      .collect();
    candles_map.insert(date.clone(), candles);
  }
  // build combinations
  let combination_results: Vec<(f64, BacktestContext)> = vec![];
  let combination_results = Arc::new(Mutex::new(combination_results));
  let combinations = build_combinations();
  let num_combinations = combinations.len();
  // backtest combinations in paralell
  let start = std::time::Instant::now();
  combinations.par_iter().for_each(|combination| {
    let (compounded_profit_loss_percentage, _trade_results) = backtest_dates(combination, &dates, &candles_map);
    if compounded_profit_loss_percentage >= 0.01 {
      log::info!("{:.4} {:?}", compounded_profit_loss_percentage, combination);
    }
    let mut combination_results = combination_results.lock().unwrap();
    combination_results.push((compounded_profit_loss_percentage, combination.clone()));
    let num_tested = combination_results.len();
    drop(combination_results);
    if num_tested % 1000 == 0 {
      let elapsed_ms = start.elapsed().as_millis();
      let elapsed_sec = start.elapsed().as_secs();
      let per_sec = (num_tested as f64 / elapsed_ms as f64) * 1000.0;
      let num_left = num_combinations - num_tested;
      let eta_seconds = num_left as f64 / per_sec;
      let percent_complete = (num_tested as f64 / num_combinations as f64) * 100.0;
      log::info!(
        "{}/{}: {:.0}/sec {}s elapsed {:.0}s eta {:.0}%",
        num_tested,
        num_combinations,
        per_sec,
        elapsed_sec,
        eta_seconds,
        percent_complete
      );
    }
  });
  let mut combination_results = combination_results.lock().unwrap();
  combination_results.sort_by(|a, b| {
    let a_compounded_profit_loss_percentage = a.0;
    let b_compounded_profit_loss_percentage = b.0;
    return b_compounded_profit_loss_percentage.partial_cmp(&a_compounded_profit_loss_percentage).unwrap();
  });
  let best_combination = &combination_results[0];
  log::info!("{}", serde_json::to_string(&serde_json::json!({
    "start_date": dates_start,
    "end_date": dates_end,
    "return": math::round(best_combination.0, 4),
    "configuration": best_combination.1
  })).unwrap());
  let backtest_context = &best_combination.1;
  let (_compounded_profit_loss_percentage, trade_results) = backtest_dates(backtest_context, &dates, &candles_map);
  log::info!("start_timestamp,direction,open_price,open_atr,outcome,exit_timestamp,exit_price,profit_loss,profit_loss_percentage,outcome_type,atr_type");
  for trade_result in &trade_results {
    debug_trade_result(trade_result);
  }
}
