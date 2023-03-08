use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{json_time};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
  Long,
  Short,
  Flat,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignalSnapshot {
  pub candle: Candle,
  pub direction: Direction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MinimalSnapshot {
  // categorize
  pub source: String,
  pub symbol: String,
  #[serde(with = "json_time")]
  pub expiration_date: NaiveDateTime,
  #[serde(with = "json_time")]
  pub scraped_at: NaiveDateTime,
  pub strike_price: f64,
  // call
  pub call_delta: f64,
  pub call_gamma: f64,
  pub call_implied_volatility: f64,
  pub call_last_trade_price: f64,
  pub call_mark_price: f64,
  pub call_open_interest: u32,
  pub call_rho: f64,
  pub call_theta: f64,
  pub call_vega: f64,
  pub call_vanna: f64,
  pub call_vomma: f64,
  pub call_charm: f64,
  pub call_volume: u32,
  // put
  pub put_delta: f64,
  pub put_gamma: f64,
  pub put_implied_volatility: f64,
  pub put_last_trade_price: f64,
  pub put_mark_price: f64,
  pub put_open_interest: u32,
  pub put_rho: f64,
  pub put_theta: f64,
  pub put_vega: f64,
  pub put_vanna: f64,
  pub put_vomma: f64,
  pub put_charm: f64,
  pub put_volume: u32,
  // underlying
  pub underlying_last_trade_price: f64,
  pub underlying_mark_price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candle {
  //pub symbol: String,
  //pub resolution: String,
  pub timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuoteSnapshot {
  //pub symbol: String, // TODO: do not hardcode?
  pub scraped_at: i64,
  pub ask_price: f64,
  pub bid_price: f64,
  pub last_trade_price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CandleSnapshot {
  //pub symbol: String,
  //pub resolution: String,
  pub scraped_at: i64,
  pub timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BacktestOutcome {
  ProfitLimit,
  StopLoss,
  DirectionChange,
}

#[derive(Serialize, Clone)]
pub struct DirectionSnapshot {
  pub timestamp: i64,
  pub direction: Direction,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Trade {
  pub start_timestamp: i64,
  pub end_timestamp: i64,
  pub formatted_start_timestamp: String,
  pub formatted_end_timestamp: String,
  pub direction: Direction,
}

#[derive(Debug, Serialize, Clone)]
pub struct TradeGenerationContext {
  pub fast_periods: usize,
  pub slow_periods: usize,
  pub warmup_periods: usize
}

impl Default for TradeGenerationContext {
  fn default() -> Self {
    Self {
      fast_periods: 9,
      slow_periods: 21,
      warmup_periods: 1
    }
  }
}

#[derive(Debug, Serialize, Clone)]
pub struct BacktestContext {
  pub slippage_percentage: f64,
  pub stop_loss_percentage: f64,
  pub profit_limit_percentage: f64,
}

impl Default for BacktestContext {
  fn default() -> Self {
    Self {
      slippage_percentage: 0.000125,
      stop_loss_percentage: -0.004,
      profit_limit_percentage: 0.004
    }
  }
}

#[derive(Debug, Serialize)]
pub struct CombinationBacktestResult {
  pub trade_generation_context: TradeGenerationContext,
  pub backtest_context: BacktestContext,
  pub num_trades: usize,
  pub simple_profit_loss_percentage: f64,
  pub compounded_profit_loss_percentage: f64
}