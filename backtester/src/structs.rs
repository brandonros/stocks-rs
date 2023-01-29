use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Direction {
  Long,
  Short,
  Flat,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum BacktestOutcome {
  ProfitLimit,
  StopLoss,
  DirectionChange,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum BacktestMode {
  SingleEntry,
  MultipleEntry,
}

#[derive(Serialize, Clone, Debug)]
pub struct DirectionChange {
  pub start_snapshot_index: usize,
  pub end_snapshot_index: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candle {
  pub timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolygonResult {
  pub c: f64,
  pub h: f64,
  pub l: f64,
  pub n: Option<i64>,
  pub o: f64,
  pub t: i64,
  pub v: f64,
  pub vw: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolygonResponseRoot {
  pub adjusted: bool,
  #[serde(rename = "queryCount")]
  pub query_count: i64,
  pub request_id: String,
  pub results: Vec<PolygonResult>,
  #[serde(rename = "resultsCount")]
  pub results_count: i64,
  pub status: String,
  pub ticker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinnhubStockCandlesResponse {
  pub c: Vec<f64>,
  pub h: Vec<f64>,
  pub l: Vec<f64>,
  pub o: Vec<f64>,
  pub t: Vec<i64>,
  pub v: Vec<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignalSnapshot {
  pub candle: Candle,
  pub direction: Direction,
}

#[derive(Serialize, Clone, Debug)]
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
pub struct BacktestStatistics {
  pub total_profit_percentage: f64,
  pub total_win_profit_percentage: f64,
  pub total_loss_profit_percentage: f64,
  pub num_trades: usize,
  pub num_dates: usize,
  pub num_trades_per_day: f64,
  pub num_profit_limits: usize,
  pub num_stop_losses: usize,
  pub num_direction_changes: usize,
  pub num_wins: usize,
  pub num_losses: usize,
  pub num_breakevens: usize,
  pub win_loss_ratio: f64,
  pub win_rate_percentage: f64,
  pub starting_portfolio_value: f64,
  pub final_portfolio_value: f64,
  pub portfolio_value_change: f64,
  pub portfolio_value_change_percentage: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct BacktestSettings {
  pub slippage_percentage: f64,
  pub profit_limit_percentage: f64,
  pub stop_loss_percentage: f64,
  pub warmed_up_index: usize,
  pub backtest_mode: BacktestMode,
}

#[derive(Serialize, Clone, Debug)]
pub struct TradePerformanceSnapshot {
  pub signal_snapshot_index: usize,
  pub peak_price: f64,
  pub trough_price: f64,
  pub base_case_scenario_exit_price: f64,
  pub best_case_scenario_profit_loss: f64,
  pub best_case_scenario_profit_loss_percentage: f64,
  pub worst_case_scenario_exit_price: f64,
  pub worst_case_scenario_profit_loss: f64,
  pub worst_case_scenario_profit_loss_percentage: f64,
}
