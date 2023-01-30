use common::structs::*;
use strategies::{supertrend::*, vwap_mvwap_ema_crossover::*, *};

use crate::math;
use crate::structs::*;

use super::trade_performance;

pub fn build_signal_snapshots_from_candles(strategy: &Strategy, indicator_settings: &StrategyIndicatorSettings, candles: &Vec<Candle>) -> Vec<SignalSnapshot> {
  let signal_snapshots = match strategy {
    Strategy::Supertrend => {
      let strategy = SupertrendStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::Supertrend(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(indicator_settings, candles)
    }
    Strategy::VwapMvwapEmaCrossover => {
      let strategy = VwapMvwapEmaCrossoverStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::VwapMvwapEmaCrossover(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(indicator_settings, candles)
    }
  };
  return signal_snapshots;
}

pub fn backtest_trade_performance_snapshots(
  trade_performance_snapshots: &Vec<TradePerformanceSnapshot>,
  signal_snapshots: &Vec<SignalSnapshot>,
  backtest_settings: &BacktestSettings,
) -> BacktestResult {
  // settings
  let slippage_percentage = backtest_settings.slippage_percentage;
  let profit_limit_percentage = backtest_settings.profit_limit_percentage;
  let stop_loss_percentage = backtest_settings.stop_loss_percentage;
  // first snapshot
  let first_snapshot = &signal_snapshots[0];
  // trade direction
  let trade_direction = first_snapshot.direction;
  // trade open price with slippage
  let open_price = math::calculate_open_price_with_slippage(trade_direction, first_snapshot.candle.open, slippage_percentage);
  // determine peak + trough snapshots
  let mut trade_peak_performance_snapshot = &trade_performance_snapshots[0];
  let mut trade_trough_performance_snapshot = &trade_performance_snapshots[0];
  let mut stop_loss_performance_snapshot = None;
  let mut profit_limit_performance_snapshot = None;
  for trade_performance_snapshot in trade_performance_snapshots {
    if trade_performance_snapshot.best_case_scenario_profit_loss_percentage > trade_peak_performance_snapshot.best_case_scenario_profit_loss_percentage {
      trade_peak_performance_snapshot = trade_performance_snapshot;
    }
    if trade_performance_snapshot.worst_case_scenario_profit_loss_percentage < trade_trough_performance_snapshot.worst_case_scenario_profit_loss_percentage {
      trade_trough_performance_snapshot = trade_performance_snapshot;
    }
    if stop_loss_performance_snapshot.is_none() && trade_performance_snapshot.worst_case_scenario_profit_loss_percentage <= stop_loss_percentage {
      stop_loss_performance_snapshot.replace(trade_performance_snapshot);
    }
    if profit_limit_performance_snapshot.is_none() && trade_performance_snapshot.best_case_scenario_profit_loss_percentage >= profit_limit_percentage {
      profit_limit_performance_snapshot.replace(trade_performance_snapshot);
    }
  }
  // calculate direction change/trade end
  let trade_end_performance_snapshot = &trade_performance_snapshots[trade_performance_snapshots.len() - 1];
  let (trade_outcome, trade_exit_performance_snapshot) = trade_performance::determine_trade_outcome(
    signal_snapshots,
    stop_loss_performance_snapshot,
    profit_limit_performance_snapshot,
    trade_end_performance_snapshot,
  );
  let trade_peak_performance_snapshot = trade_peak_performance_snapshot;
  let trade_trough_performance_snapshot = trade_trough_performance_snapshot;
  let trade_peak_signal_snapshot = &signal_snapshots[trade_peak_performance_snapshot.signal_snapshot_index];
  let trade_trough_signal_snapshot = &signal_snapshots[trade_trough_performance_snapshot.signal_snapshot_index];
  let trade_exit_signal_snapshot = &signal_snapshots[trade_exit_performance_snapshot.signal_snapshot_index];
  // calculate profit limit + stop loss price from open + direction
  let profit_limit_price = math::calculate_profit_limit_price(trade_direction, open_price, profit_limit_percentage);
  let stop_loss_price = math::calculate_stop_loss_price(trade_direction, open_price, stop_loss_percentage);
  if trade_direction == Direction::Long {
    assert!(profit_limit_price > open_price);
    assert!(stop_loss_price < open_price);
  } else {
    assert!(profit_limit_price < open_price);
    assert!(stop_loss_price > open_price);
  }
  let exit_price = if trade_outcome == BacktestOutcome::StopLoss {
    math::calculate_close_price_with_slippage(trade_direction, stop_loss_price, slippage_percentage)
  } else if trade_outcome == BacktestOutcome::ProfitLimit {
    math::calculate_close_price_with_slippage(trade_direction, profit_limit_price, slippage_percentage)
  } else {
    math::calculate_close_price_with_slippage(trade_direction, trade_exit_signal_snapshot.candle.close, slippage_percentage)
  };
  // profit loss
  let profit_loss = math::calculate_profit_loss(trade_direction, open_price, exit_price);
  let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
  // duration
  let trade_duration = trade_exit_signal_snapshot.candle.timestamp - first_snapshot.candle.timestamp;
  return BacktestResult {
    outcome: trade_outcome,
    trade_entry_snapshot: first_snapshot.clone(),                // TODO: get rid of clone?
    trade_peak_snapshot: trade_peak_signal_snapshot.clone(),     // TODO: get rid of clone?
    trade_trough_snapshot: trade_trough_signal_snapshot.clone(), // TODO: get rid of clone?
    trade_exit_snapshot: trade_exit_signal_snapshot.clone(),     // TODO: get rid of clone?
    trade_peak_profit_loss_percentage: trade_peak_performance_snapshot.best_case_scenario_profit_loss_percentage,
    trade_trough_profit_loss_percentage: trade_trough_performance_snapshot.worst_case_scenario_profit_loss_percentage,
    trade_duration,
    open_price,
    profit_limit_price,
    stop_loss_price,
    exit_price,
    profit_loss,
    profit_loss_percentage,
  };
}
