use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{database, json_time};

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

impl database::ToQuery for Candle {
  fn insert(&self) -> (&str, Vec<(&str, &dyn rusqlite::ToSql)>) {
    let query = "
        INSERT OR REPLACE INTO candles (
          symbol,
          resolution,
          scraped_at,
          timestamp,
          open,
          high,
          low,
          close,
          volume
      ) VALUES (
          :symbol,
          :resolution,
          strftime('%s', 'now'),
          :timestamp,
          :open,
          :high,
          :low,
          :close,
          :volume
      )
    ";
    let params = rusqlite::named_params! {
      ":symbol": "SPY", // TODO: hardcoded this to get rid of .clone everywhere
      ":resolution": "1", // TODO: hardcoded this to get rid of .clone everywhere
      ":timestamp": self.timestamp,
      ":open": self.open,
      ":high": self.high,
      ":low": self.low,
      ":close": self.close,
      ":volume": self.volume
    };
    return (query, params.to_vec());
  }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BacktestResult {
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_limit_price: f64,
  pub stop_loss_price: f64,
  pub outcome: BacktestOutcome,
  pub trade_entry_snapshot: SignalSnapshot,
  pub trade_peak_snapshot: SignalSnapshot,
  pub trade_trough_snapshot: SignalSnapshot,
  pub trade_exit_snapshot: SignalSnapshot,
  pub trade_peak_profit_loss_percentage: f64,
  pub trade_trough_profit_loss_percentage: f64,
  pub trade_duration: i64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ReducedBacktestResult {
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_limit_price: f64,
  pub stop_loss_price: f64,
  pub outcome: BacktestOutcome,
  pub trade_entry_snapshot: SignalSnapshot,
  pub trade_exit_snapshot: SignalSnapshot,
  pub trade_duration: i64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BacktestStatistic {
  pub compounded_profit_loss_percentage: f64,
  pub profit_loss_percentage: f64,
  pub profit_loss_percentage_from_losses: f64,
  pub profit_loss_percentage_from_wins: f64,
  pub profit_loss_percentage_from_direction_change_losses: f64,
  pub profit_loss_percentage_from_direction_change_wins: f64,
  pub profit_loss_percentage_from_long: f64,
  pub profit_loss_percentage_from_short: f64,
  pub num_trades: usize,
  pub num_days: usize,
  pub num_wins: usize,
  pub num_losses: usize,
  pub num_direction_changes: usize,
  pub num_winning_direction_changes: usize,
  pub num_losing_direction_changes: usize,
  pub num_flat_direction_changes: usize,
  pub num_long: usize,
  pub num_long_wins: usize,
  pub num_long_losses: usize,
  pub num_long_direction_changes: usize,
  pub num_short: usize,
  pub num_short_wins: usize,
  pub num_short_losses: usize,
  pub num_short_direction_changes: usize,
}


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BacktestCombination {
  pub supertrend_periods: usize,
  pub supertrend_multiplier: f64,
  pub profit_limit_percentage: f64,
  pub stop_loss_percentage: f64,
  pub warmed_up_index: usize,
}