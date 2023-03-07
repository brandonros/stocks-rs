use std::{collections::{HashMap}};

use serde::Serialize;

use crate::{math, structs::*, dates};

#[derive(PartialEq, Serialize, Debug)]
pub enum Outcome {
  StopLoss,
  ProfitLimit,
  DirectionChange,
}

#[derive(Serialize, Debug)]
pub struct TradeBacktestResult {
  pub direction: Direction,
  pub start_timestamp: i64,
  pub exit_timestamp: i64,
  pub formatted_start_timestamp: String,
  pub formatted_exit_timestamp: String,
  pub outcome: Outcome,
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

fn calculate_trade_result(
  backtest_context: &BacktestContext,
  trade_candles: &Vec<Candle>,
  trade_direction: &Direction,
  start_timestamp: i64,
  open_price: f64,
) -> TradeBacktestResult {
  // backtest context variables
  let slippage_percentage = backtest_context.slippage_percentage;
  let stop_loss_percentage = backtest_context.stop_loss_percentage;
  let profit_limit_percentage = backtest_context.profit_limit_percentage;
  // stop loss/profit limit prices
  let stop_loss_price = math::calculate_stop_loss_price(trade_direction, open_price, stop_loss_percentage);
  let profit_limit_price = math::calculate_profit_limit_price(trade_direction, open_price, profit_limit_percentage);
  for i in 0..trade_candles.len() {
    let trade_candle = &trade_candles[i];
    // worst case scenario first based on direction for stop loss
    let exit_price = if *trade_direction == Direction::Long {
      trade_candle.low
    } else {
      trade_candle.high
    };
    let open_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
    let stop_loss_hit = open_profit_loss_percentage <= stop_loss_percentage;
    if stop_loss_hit {
      // force exit price to be capped to exactly a perfect fill stop_loss_price at worse
      let exit_price = stop_loss_price;
      let profit_loss = math::calculate_profit_loss(trade_direction, open_price, exit_price);
      let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
      return TradeBacktestResult {
        open_price,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        formatted_start_timestamp: dates::format_timestamp(start_timestamp),
        formatted_exit_timestamp: dates::format_timestamp(trade_candle.timestamp),
        outcome: Outcome::StopLoss,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
    // best case scenario next based on direction for profit limit
    let exit_price = if *trade_direction == Direction::Long {
      trade_candle.high
    } else {
      trade_candle.low
    };
    let open_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
    let profit_limit_hit = open_profit_loss_percentage >= profit_limit_percentage;
    if profit_limit_hit {
      // force exit price to be capped to exactly a perfect fill profit_limit_price at best
      let exit_price = profit_limit_price;
      let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, exit_price);
      let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
      return TradeBacktestResult {
        open_price,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        formatted_start_timestamp: dates::format_timestamp(start_timestamp),
        formatted_exit_timestamp: dates::format_timestamp(trade_candle.timestamp),
        outcome: Outcome::ProfitLimit,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
  }
  // exit on last candle close (TODO: this is probably unrealistic and it'd be the next candle open that happens 1 second later (on registered direction change))
  let trade_end_candle = &trade_candles[trade_candles.len() - 1];
  let exit_price = trade_end_candle.close;
  let exit_price = math::calculate_close_price_with_slippage(&trade_direction, exit_price, slippage_percentage);
  let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, exit_price);
  let profit_loss_percentage = math::calculate_profit_loss_percentage(&trade_direction, open_price, exit_price);
  // direction change within range of stop loss and profit limit
  return TradeBacktestResult {
    open_price,
    direction: trade_direction.clone(),
    start_timestamp,
    exit_timestamp: trade_end_candle.timestamp,
    formatted_start_timestamp: dates::format_timestamp(start_timestamp),
    formatted_exit_timestamp: dates::format_timestamp(trade_end_candle.timestamp),
    outcome: Outcome::DirectionChange,
    exit_price,
    profit_loss,
    profit_loss_percentage,
  };
}

pub fn generate_trades_results(backtest_context: &BacktestContext, trades: &Vec<Trade>, candles: &Vec<Candle>) -> Vec<TradeBacktestResult> {
  let mut trade_results: Vec<TradeBacktestResult> = vec![];
  for trade in trades {
    // skip flat signals
    if trade.direction == Direction::Flat {
      continue;
    }
    // get candles in range
    let trade_candles: Vec<Candle> = candles
      .iter()
      .filter(|candle| {
        return candle.timestamp >= trade.start_timestamp && candle.timestamp <= trade.end_timestamp;
      })
      .cloned()
      .collect();
    // watch out for empty trades?
    if trade_candles.len() == 0 {
      log::warn!("no trade candles? trade = {:?}", trade);
      continue;
    }
    // get open price
    let open_candle = &trade_candles[0];
    let slippage_percentage = backtest_context.slippage_percentage;
    let open_price = math::calculate_open_price_with_slippage(&trade.direction, open_candle.open, slippage_percentage);
    // get trade result
    let trade_result = calculate_trade_result(&backtest_context, &trade_candles, &trade.direction, open_candle.timestamp, open_price);
    trade_results.push(trade_result);
  }
  return trade_results;
}

pub fn generate_dates_trades_results_map(
  dates: &Vec<String>,
  backtest_context: &BacktestContext,
  candles_date_map: &HashMap<String, Vec<Candle>>,
  dates_trades_map: &HashMap<String, Vec<Trade>>,
) -> HashMap<String, Vec<TradeBacktestResult>> {
  let mut dates_trades_results_map = HashMap::new();
  for date in dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let date_trades = dates_trades_map.get(date).unwrap();
    let date_trade_results = generate_trades_results(backtest_context, date_trades, date_candles);
    dates_trades_results_map.insert(date.clone(), date_trade_results);
  }
  return dates_trades_results_map;
}
