use crate::structs::*;
use common::math;
use common::structs::*;
use strategies::*;

pub fn build_trade_performance_snapshots_from_direction_changes(
  direction_changes: &Vec<DirectionChange>,
  signal_snapshots: &Vec<SignalSnapshot>,
  slippage_percentage: f64,
) -> Vec<Vec<TradePerformanceSnapshot>> {
  let mut results = vec![];
  for direction_change in direction_changes {
    let start_snapshot_index = direction_change.start_snapshot_index;
    let end_snapshot_index = direction_change.end_snapshot_index.unwrap();
    let trade_signal_snapshots = &signal_snapshots[start_snapshot_index..end_snapshot_index].to_vec(); // TODO: get rid of clone?
    if trade_signal_snapshots.len() == 0 {
      // log::warn!("trade_signal_snapshots.len() == 0");
      continue;
    }
    let first_trade_snapshot = &trade_signal_snapshots[0];
    let trade_direction = first_trade_snapshot.direction;
    let open_price = math::calculate_open_price_with_slippage(trade_direction, first_trade_snapshot.candle.open, slippage_percentage);
    let trade_performance_snapshots = build_trade_performance_from_trade_snapshots(trade_direction, &trade_signal_snapshots, open_price);
    results.push(trade_performance_snapshots);
  }
  return results;
}

pub fn build_trade_performance_from_trade_snapshots(
  trade_direction: Direction,
  signal_snapshots: &Vec<SignalSnapshot>,
  open_price: f64,
) -> Vec<TradePerformanceSnapshot> {
  // TODO: skip same candle we open trade on?
  return signal_snapshots
    .iter()
    .enumerate()
    .map(|(index, signal_snapshot)| {
      let peak_price = signal_snapshot.candle.high;
      let trough_price = signal_snapshot.candle.low;
      // profit limit
      let base_case_scenario_exit_price = if trade_direction == Direction::Long { peak_price } else { trough_price };
      let best_case_scenario_profit_loss = math::calculate_profit_loss(trade_direction, open_price, base_case_scenario_exit_price);
      let best_case_scenario_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, base_case_scenario_exit_price);
      // stop loss
      let worst_case_scenario_exit_price = if trade_direction == Direction::Long { trough_price } else { peak_price };
      let worst_case_scenario_profit_loss = math::calculate_profit_loss(trade_direction, open_price, worst_case_scenario_exit_price);
      let worst_case_scenario_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, worst_case_scenario_exit_price);
      return TradePerformanceSnapshot {
        signal_snapshot_index: index,
        peak_price,
        trough_price,
        base_case_scenario_exit_price,
        best_case_scenario_profit_loss,
        best_case_scenario_profit_loss_percentage,
        worst_case_scenario_exit_price,
        worst_case_scenario_profit_loss,
        worst_case_scenario_profit_loss_percentage,
      };
    })
    .collect();
}

pub fn determine_trade_outcome<'a>(
  signal_snapshots: &Vec<SignalSnapshot>,
  stop_loss_performance_snapshot: Option<&TradePerformanceSnapshot>,
  profit_limit_performance_snapshot: Option<&TradePerformanceSnapshot>,
  trade_end_performance_snapshot: &TradePerformanceSnapshot,
) -> (BacktestOutcome, TradePerformanceSnapshot) {
  if stop_loss_performance_snapshot.is_none() && profit_limit_performance_snapshot.is_none() {
    return (BacktestOutcome::DirectionChange, trade_end_performance_snapshot.clone());
    // TODO: get rid of clone?
  }
  if stop_loss_performance_snapshot.is_some() && profit_limit_performance_snapshot.is_none() {
    return (BacktestOutcome::StopLoss, stop_loss_performance_snapshot.unwrap().clone());
    // TODO: get rid of clone?
  }
  if stop_loss_performance_snapshot.is_none() && profit_limit_performance_snapshot.is_some() {
    return (BacktestOutcome::ProfitLimit, profit_limit_performance_snapshot.unwrap().clone());
    // TODO: get rid of clone?
  }
  // assumes both a potential stop loss and profit limit happened, check which came first time wise
  let stop_loss_performance_snapshot = stop_loss_performance_snapshot.unwrap();
  let profit_limit_performance_snapshot = profit_limit_performance_snapshot.unwrap();
  let stop_loss_signal_snapshot = &signal_snapshots[stop_loss_performance_snapshot.signal_snapshot_index];
  let profit_limit_signal_snapshot = &signal_snapshots[profit_limit_performance_snapshot.signal_snapshot_index];
  if stop_loss_signal_snapshot.candle.timestamp <= profit_limit_signal_snapshot.candle.timestamp {
    return (BacktestOutcome::StopLoss, stop_loss_performance_snapshot.clone()); // TODO: get rid of clone?
  } else {
    return (BacktestOutcome::ProfitLimit, profit_limit_performance_snapshot.clone());
    // TODO: get rid of clone?
  }
}
