use crate::math;
use crate::structs::*;

pub fn calculate_backtest_statistics(num_dates: usize, backtest_results: &Vec<BacktestResult>) -> BacktestStatistics {
  let num_trades = backtest_results.len();
  let total_profit_percentage = backtest_results.iter().fold(0.0, |prev, result| {
    return prev + result.profit_loss_percentage;
  });
  let total_win_profit_percentage = backtest_results.iter().fold(0.0, |prev, result| {
    if result.profit_loss_percentage <= 0.0 {
      return prev;
    }
    return prev + result.profit_loss_percentage;
  });
  let total_loss_profit_percentage = backtest_results.iter().fold(0.0, |prev, result| {
    if result.profit_loss_percentage >= 0.0 {
      return prev;
    }
    return prev + result.profit_loss_percentage;
  });
  let num_wins = backtest_results.iter().fold(0, |prev, result| {
    if result.profit_loss_percentage > 0.0 {
      return prev + 1;
    }
    return prev;
  });
  let num_losses = backtest_results.iter().fold(0, |prev, result| {
    if result.profit_loss_percentage < 0.0 {
      return prev + 1;
    }
    return prev;
  });
  let num_breakevens = backtest_results.iter().fold(0, |prev, result| {
    if result.profit_loss_percentage == 0.0 {
      return prev + 1;
    }
    return prev;
  });
  let win_loss_ratio = num_wins as f64 / num_losses as f64;
  let win_rate_percentage = num_wins as f64 / num_trades as f64;
  let num_trades_per_day = num_trades as f64 / num_dates as f64;
  let num_profit_limits = backtest_results.iter().fold(0, |prev, backtest_result| {
    if backtest_result.outcome == BacktestOutcome::ProfitLimit {
      return prev + 1;
    }
    return prev;
  });
  let num_stop_losses = backtest_results.iter().fold(0, |prev, backtest_result| {
    if backtest_result.outcome == BacktestOutcome::StopLoss {
      return prev + 1;
    }
    return prev;
  });
  let num_direction_changes = backtest_results.iter().fold(0, |prev, backtest_result| {
    if backtest_result.outcome == BacktestOutcome::DirectionChange {
      return prev + 1;
    }
    return prev;
  });
  let starting_portfolio_value = 1000.00;
  let final_portfolio_value = backtest_results.iter().fold(starting_portfolio_value, |prev, backtest_result| {
    let result = prev * (1.0 + backtest_result.profit_loss_percentage);
    //let difference = result - prev;
    //log::info!("{},{},{},{},{}", backtest_result.trade_entry_snapshot.candle.timestamp, backtest_result.trade_exit_snapshot.candle.timestamp, prev, result, difference);
    return result;
  });
  let portfolio_value_change = final_portfolio_value - starting_portfolio_value;
  let portfolio_value_change_percentage = math::calculate_percentage_increase(starting_portfolio_value, final_portfolio_value);
  return BacktestStatistics {
    total_profit_percentage,
    total_win_profit_percentage,
    total_loss_profit_percentage,
    num_trades,
    num_dates,
    num_trades_per_day,
    num_profit_limits,
    num_stop_losses,
    num_direction_changes,
    num_wins,
    num_losses,
    num_breakevens,
    win_loss_ratio,
    win_rate_percentage,
    starting_portfolio_value,
    final_portfolio_value,
    portfolio_value_change,
    portfolio_value_change_percentage,
  };
}
