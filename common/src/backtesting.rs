use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{math, structs::*};

#[derive(Serialize, Debug)]
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
  pub outcome: Outcome,
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

fn calculate_trade_result(
  backtest_context: &BacktestContext,
  trade_candles: &Vec<Arc<Candle>>,
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
    outcome: Outcome::DirectionChange,
    exit_price,
    profit_loss,
    profit_loss_percentage,
  };
}

/*fn debug_trade_result(trade_result: &TradeResult) {
  let trade_result_type = if trade_result.profit_loss > 0.0 {
    String::from("win")
  } else {
    String::from("loss")
  };
  let mut row = vec![];
  row.push(format!("{}", dates::format_timestamp(trade_result.start_timestamp)));
  row.push(format!("{:?}", trade_result.direction));
  row.push(format!("${:.2}", trade_result.open_price));
  row.push(format!("{:?}", trade_result.outcome));
  row.push(format!("{}", dates::format_timestamp(trade_result.exit_timestamp)));
  row.push(format!("${:.2}", trade_result.exit_price));
  row.push(format!("${:.2}", trade_result.profit_loss));
  row.push(format!("{:.4}", trade_result.profit_loss_percentage));
  row.push(trade_result_type);
  log::info!("{}", row.join(","));
}*/

pub fn generate_dates_trades_results_map(
  dates: &Vec<String>,
  backtest_context: &BacktestContext,
  candles_date_map: &HashMap<String, Vec<Arc<Candle>>>,
  dates_trades_map: &HashMap<String, Vec<Trade>>,
) -> HashMap<String, Vec<TradeBacktestResult>> {
  let mut dates_trades_results_map = HashMap::new();
  for date in dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let date_trades = dates_trades_map.get(date).unwrap();
    let date_trade_results: Vec<TradeBacktestResult> = date_trades
      .iter()
      .map(|trade| {
        let trade_candles: Vec<Arc<Candle>> = date_candles
          .iter()
          .filter(|candle| {
            return candle.timestamp >= trade.start_timestamp && candle.timestamp <= trade.end_timestamp;
          })
          .cloned()
          .collect();
        let open_candle = &trade_candles[0];
        let slippage_percentage = backtest_context.slippage_percentage;
        let open_price = math::calculate_open_price_with_slippage(&trade.direction, open_candle.open, slippage_percentage);
        return calculate_trade_result(&backtest_context, &trade_candles, &trade.direction, open_candle.timestamp, open_price);
      })
      .collect();
    dates_trades_results_map.insert(date.clone(), date_trade_results);
  }
  return dates_trades_results_map;
}
