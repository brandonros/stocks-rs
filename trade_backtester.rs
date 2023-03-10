#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! csv = "1.2.1"
//! serde = { version = "1.0.153", features = ["derive"] }
//! ```

use std::collections::HashMap;
use std::fs::File;
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct Candle {
  pub start_timestamp: i64,
  pub end_timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

#[derive(PartialEq, Debug, Deserialize, Clone)]
enum Direction {
  Long,
  Short,
  Flat,
}

#[derive(PartialEq, Deserialize)]
enum TradeType {
  Open,
  Close
}

#[derive(Deserialize)]
struct Trade {
  pub timestamp: i64,
  pub r#type: TradeType,
  pub direction: Direction
}

#[derive(Debug)]
enum TradeExitType {
  Win,
  Loss
}

#[derive(Debug)]
enum TradeExitReason {
  StopLoss,
  ProfitLimit,
  Close
}

struct TradeBacktestResult {
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
  exit_type: TradeExitType
}

fn read_records_from_csv<T>(filename: &str) -> Vec<T>
where
  T: for<'de> Deserialize<'de>{
  let mut candles = vec![];
  let file = File::open(filename).unwrap();
  let mut csv_reader = ReaderBuilder::new()
    .has_headers(true)
    .from_reader(file);
  for record in csv_reader.deserialize() {
    let candle: T = record.unwrap();
    candles.push(candle);
  }
  return candles;
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

fn backtest_trade(trade_open: &Trade, trade_close: &Trade, candles_map: &HashMap<i64, Candle>) -> TradeBacktestResult {
  let slippage_percentage = 0.000125;
  let profit_limit_percentage = 0.004;
  let stop_loss_percentage = -0.004;
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
    let candle_size_seconds = 300; // TODO: do not hardcode
    let mut pointer = trade_open.timestamp;
    while pointer <= trade_close.timestamp {
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
    return (TradeExitReason::Close, close_price, close_candle);
  };
  let (exit_reason, exit_price, exit_candle) = determine_trade_exit();
  let profit_loss = if trade_open.direction == Direction::Long {
    exit_price - open_price
  } else {
    open_price - exit_price
  };
  let profit_loss_percentage = profit_loss / open_price;
  let exit_type = if profit_loss > 0.0 {
    TradeExitType::Win
  } else {
    TradeExitType::Loss
  };
  return TradeBacktestResult {
    open_timestamp: trade_open.timestamp,
    exit_timestamp: exit_candle.start_timestamp,
    close_timestamp: trade_close.timestamp,
    open_price,
    close_price,
    profit_limit_price,
    stop_loss_price,
    exit_reason,
    exit_candle: exit_candle.clone(),
    exit_price,
    profit_loss,
    profit_loss_percentage,
    exit_type
  };
}

fn main() {
  // load candles
  let candles_filename = std::env::args().nth(1).unwrap();
  let candles = read_records_from_csv::<Candle>(&candles_filename);
  let mut candles_map = HashMap::new();
  for candle in candles {
    candles_map.insert(candle.start_timestamp, candle);
  }
  // load trades
  let trades_filename = std::env::args().nth(2).unwrap();
  let trades = read_records_from_csv::<Trade>(&trades_filename);
  // print header
  println!("open_timestamp,exit_timestamp,direction,open_price,exit_price,profit_loss,profit_loss_percentage,exit_reason,exit_type");
  // chunk trades
  let trades_slice: &[Trade] = &trades;
  let chunked_trades: Vec<&[Trade]> = trades_slice.chunks(2).collect();
  for chunk in chunked_trades {
    // get open + close
    let trade_open = &chunk[0];
    let trade_close = &chunk[1];
    assert!(trade_open.r#type == TradeType::Open);
    assert!(trade_close.r#type == TradeType::Close);    
    assert!(trade_open.direction == trade_close.direction);
    assert!(trade_open.timestamp != trade_close.timestamp);
    // backtest trade
    let result = backtest_trade(&trade_open, &trade_close, &candles_map);
    println!(
      "{open_timestamp},{exit_timestamp},{direction:?},{open_price:.2},{exit_price:.2},{profit_loss:.2},{profit_loss_percentage:.4},{exit_reason:?},{exit_type:?}", 
      open_timestamp = result.open_timestamp,
      exit_timestamp = result.exit_timestamp,
      direction = trade_open.direction,
      open_price = result.open_price,
      exit_price = result.exit_price,
      profit_loss = result.profit_loss,
      profit_loss_percentage = result.profit_loss_percentage,
      exit_reason = result.exit_reason,
      exit_type = result.exit_type
    );
  }
}
