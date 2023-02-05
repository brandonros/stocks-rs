use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum BacktestMode {
  SingleEntry,
  MultipleEntry,
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
