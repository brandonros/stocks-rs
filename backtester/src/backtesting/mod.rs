use std::collections::HashMap;

use common::database::Database;
use common::structs::*;
use providers::Provider;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use strategies::*;
use tokio::io::AsyncWriteExt;

use crate::{market_session, math, signals, structs::*};

pub mod combinations;
pub mod signal_snapshots;
pub mod statistics;
pub mod trade_performance;

pub async fn backtest(symbol: &str, resolution: &str, provider: &Provider, strategy: &Strategy, dates: &Vec<&str>) {
  // connect to databse
  let connection = Database::new(&format!("{:?}", provider));
  connection.migrate("./schema/");
  // pull candles
  log::info!("pulling candles");
  let mut candles_dates_map: HashMap<&str, Vec<Candle>> = HashMap::new();
  for date in dates {
    let (from, to) = market_session::get_regular_market_start_end_from_string(date);
    let from_timestamp = from.timestamp();
    let to_timestamp = to.timestamp();
    let query = format!(
      "SELECT * FROM candles WHERE resolution = '{resolution}' AND symbol = '{symbol}' AND timestamp >= {from_timestamp} AND timestamp <= {to_timestamp}"
    );
    let candles = connection.get_rows_from_database::<Candle>(&query);
    // TODO: check candles.len() based on timeframe? 1 = 390, 5 = 78, 15 = 26
    candles_dates_map.insert(date, candles);
  }
  log::info!("pulled candles");
  // calculate num dates due to weird holiday issue for statistics trades/num_days
  let num_dates = dates.iter().fold(0, |prev, date| {
    let date_candles = candles_dates_map.get(date).unwrap();
    if date_candles.len() == 0 {
      //log::warn!("skipping {} due to no candles", date);
      return prev;
    }
    return prev + 1;
  });
  // build combinations
  log::info!("building combinations");
  let indicator_setting_combinations = combinations::build_indicator_setting_combinations(strategy);
  let backtest_setting_combinations = combinations::build_backtest_setting_combinations();
  log::info!("built combinations");
  // build caches based on indicator setting combinations
  log::info!(
    "building caches for {} dates {} indicator settings combinations {} backtest settings combinations",
    num_dates,
    indicator_setting_combinations.len(),
    backtest_setting_combinations.len()
  );
  let mut date_indicator_settings_signal_snapshots_cache = HashMap::<String, Vec<SignalSnapshot>>::new();
  let mut date_indicator_settings_direction_changes_cache = HashMap::<String, Vec<DirectionChange>>::new();
  let mut date_indicator_settings_performance_snapshots_cache = HashMap::<String, Vec<Vec<TradePerformanceSnapshot>>>::new();
  for indicator_setting_combination in &indicator_setting_combinations {
    for date in dates {
      let date_candles = candles_dates_map.get(date).unwrap();
      if date_candles.len() == 0 {
        //log::warn!("skipping {} due to no candles", date);
        continue;
      }
      let signal_snapshots = signal_snapshots::build_signal_snapshots_from_candles(strategy, indicator_setting_combination, date_candles);
      let warmed_up_index = 10;
      let slippage_percentage = 0.00025; // TODO: do not hardcode
      let direction_changes = signals::build_direction_changes_from_signal_snapshots(&signal_snapshots, warmed_up_index);
      let direction_changes_performance_snapshots =
        trade_performance::build_trade_performance_snapshots_from_direction_changes(&direction_changes, &signal_snapshots, slippage_percentage);
      let key = format!("{}:{:?}:{:?}:{}", date, strategy, indicator_setting_combination, warmed_up_index);
      date_indicator_settings_signal_snapshots_cache.insert(key.clone(), signal_snapshots);
      date_indicator_settings_direction_changes_cache.insert(key.clone(), direction_changes);
      date_indicator_settings_performance_snapshots_cache.insert(key.clone(), direction_changes_performance_snapshots);
    }
  }
  log::info!("built caches");
  // combine combinations to allow for parallel compuations
  let mut combined_combinations = vec![];
  for indicator_setting_combination in &indicator_setting_combinations {
    for backtest_setting_combination in &backtest_setting_combinations {
      combined_combinations.push((indicator_setting_combination, backtest_setting_combination));
    }
  }
  // backtest backtest setting combinations
  log::info!("backtesting combinations");
  let num_backtested = std::sync::atomic::AtomicUsize::new(0);
  let num_combinations = combined_combinations.len();
  let mut combination_results: Vec<(&StrategyIndicatorSettings, &BacktestSettings, BacktestStatistics)> = combined_combinations
    .into_par_iter()
    .map(|(indicator_settings, backtest_settings)| {
      let mut backtest_dates_results = vec![];
      for date in dates {
        // get date candles
        let date_candles = candles_dates_map.get(date).unwrap();
        if date_candles.len() == 0 {
          //log::warn!("skipping {} due to no candles", date);
          continue;
        }
        // get cached strategy signal/direction changes by date
        let key = format!("{}:{:?}:{:?}:{}", date, strategy, indicator_settings, backtest_settings.warmed_up_index);
        let signal_snapshots = date_indicator_settings_signal_snapshots_cache.get(&key).unwrap();
        let direction_changes = date_indicator_settings_direction_changes_cache.get(&key).unwrap();
        let direction_changes_performance_snapshots = date_indicator_settings_performance_snapshots_cache.get(&key).unwrap();
        // backtest every direction change in date
        if backtest_settings.backtest_mode == BacktestMode::MultipleEntry {
          panic!("TODO");
        }
        let mut backtest_date_results = vec![];
        for (index, direction_change) in direction_changes.into_iter().enumerate() {
          let start_snapshot_index = direction_change.start_snapshot_index;
          let end_snapshot_index = direction_change.end_snapshot_index.unwrap();
          let trade_signal_snapshots = &signal_snapshots[start_snapshot_index..end_snapshot_index].to_vec(); // TODO: get rid of clone?
                                                                                                             // watch out for erroneous end of day direction change
          if trade_signal_snapshots.len() == 0 {
            //log::warn!("trade_snapshots.len() == 0 {:?}", direction_change);
            continue;
          }
          let direction_change_performance_snapshots = &direction_changes_performance_snapshots[index];
          let result = signal_snapshots::backtest_trade_performance_snapshots(direction_change_performance_snapshots, signal_snapshots, backtest_settings);
          backtest_date_results.push(result);
        }
        backtest_dates_results.push(backtest_date_results);
      }
      let flattened_backtest_results: Vec<BacktestResult> = backtest_dates_results.into_iter().flatten().collect();
      let backtest_statistics = statistics::calculate_backtest_statistics(num_dates, &flattened_backtest_results);
      let num_backtested = num_backtested.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
      if num_backtested % 1000 == 0 {
        log::info!("{} / {}", num_backtested, num_combinations);
      }
      return (indicator_settings, backtest_settings, backtest_statistics);
    })
    .collect();
  log::info!("backtested combinations");
  // do algebra
  /*log::info!("doing algebra");
  let mut profit_loss_percentages = vec![];
  let mut profit_limit_percentages = vec![];
  let mut stop_loss_percentages = vec![];
  for combination_result in &combination_results {
    let backtest_statistics = &combination_result.1;
    let backtest_settings = &combination_result.3;
    profit_loss_percentages.push(backtest_statistics.total_profit_percentage);
    profit_limit_percentages.push(backtest_settings.profit_limit_percentage);
    stop_loss_percentages.push(backtest_settings.stop_loss_percentage);
    log::info!("{},{},{}", round(backtest_statistics.total_profit_percentage, 5), round(backtest_settings.profit_limit_percentage, 5), round(backtest_settings.stop_loss_percentage, 5));
  }
  let data = vec![
    ("profit_loss_percentage", profit_loss_percentages),
    ("profit_limit_percentage", profit_limit_percentages),
    ("stop_loss_percentage", stop_loss_percentages)
  ];
  let data = linregress::RegressionDataBuilder::new().build_from(data).unwrap();
  let formula = "profit_loss_percentage ~ profit_limit_percentage + stop_loss_percentage";
  let model = linregress::FormulaRegressionBuilder::new()
      .data(&data)
      .formula(formula)
      .fit()
      .unwrap();
  log::info!("did algebra");
  let parameters: Vec<_> = model.iter_parameter_pairs().collect();
  let pvalues: Vec<_> = model.iter_p_value_pairs().collect();
  let standard_errors: Vec<_> = model.iter_se_pairs().collect();
  log::info!("{:?} {:?} {:?}", parameters, pvalues, standard_errors);*/
  // sort
  combination_results.sort_by(|a, b| {
    let a_backtest_statistics = &a.2;
    let b_backtest_statistics = &b.2;
    return b_backtest_statistics
      .portfolio_value_change_percentage
      .partial_cmp(&a_backtest_statistics.portfolio_value_change_percentage)
      .unwrap();
  });
  // print all
  for combination_result in &combination_results {
    let indicator_settings = &combination_result.0;
    let backtest_settings = &combination_result.1;
    let backtest_statistics = &combination_result.2;
    if backtest_statistics.portfolio_value_change_percentage <= 5.0 {
      continue;
    }
    // skip high trade days?
    /*if backtest_statistics.num_trades_per_day > 20.0 {
      continue;
    }*/
    log::info!(
      "{},{},{},{},{:?}",
      math::round(backtest_statistics.portfolio_value_change_percentage, 3),
      backtest_statistics.num_trades_per_day,
      math::round(backtest_settings.profit_limit_percentage, 5),
      math::round(backtest_settings.stop_loss_percentage, 5),
      indicator_settings
    );
  }
  // print best combination results
  let highest_combination_result = &combination_results[0];
  // write to file
  let stringified_results = serde_json::to_string_pretty(&combination_results).unwrap();
  let mut file = tokio::fs::File::create("/tmp/output.json").await.unwrap();
  file.write_all(stringified_results.as_bytes()).await.unwrap();
  // log to console
  log::info!("{:?}", highest_combination_result.0);
  log::info!("{:?}", highest_combination_result.1);
  log::info!("{:?}", highest_combination_result.2);
  //log::info!("{:?}", highest_combination_result.3);*/
}

#[cfg(test)]
mod tests {
  use crate::{backtesting::*, math::round};

  #[test]
  fn should_match_quantconnect_results() {
    let backtest_settings = BacktestSettings {
      slippage_percentage: 0.00025,
      profit_limit_percentage: 0.0025,
      stop_loss_percentage: -0.00125,
      warmed_up_index: 10,
      backtest_mode: BacktestMode::SingleEntry,
    };
    let stringified_trade_signal_snapshots = std::fs::read_to_string("./assets/trade-snapshots.json").unwrap();
    let trade_signal_snapshots: Vec<SignalSnapshot> = serde_json::from_str(&stringified_trade_signal_snapshots).unwrap();
    let direction_changes = vec![DirectionChange {
      start_snapshot_index: 0,
      end_snapshot_index: Some(trade_signal_snapshots.len() - 1),
    }];
    let direction_change_performance_snapshots = trade_performance::build_trade_performance_snapshots_from_direction_changes(
      &direction_changes,
      &trade_signal_snapshots,
      backtest_settings.slippage_percentage,
    );
    let backtest_result =
      signal_snapshots::backtest_trade_performance_snapshots(&direction_change_performance_snapshots[0], &trade_signal_snapshots, &backtest_settings);
    // open at 9:40am
    assert_eq!(backtest_result.trade_entry_snapshot.candle.timestamp, 1674484800);
    assert_eq!(round(backtest_result.trade_entry_snapshot.candle.open, 2), 396.25);
    // add slippage to candle open
    assert_eq!(round(backtest_result.open_price, 2), 396.35);
    // calculate profit limit + stop loss (without slippage? slippage deducted at exit price?)
    assert_eq!(round(backtest_result.profit_limit_price, 2), 397.34);
    assert_eq!(round(backtest_result.stop_loss_price, 2), 395.85);
    // determine outcome
    assert_eq!(backtest_result.outcome, BacktestOutcome::ProfitLimit);
    // determine exit from profit limit price with slippage
    assert_eq!(round(backtest_result.exit_price, 2), 397.24);
    // exit by 9:47am
    assert_eq!(backtest_result.trade_exit_snapshot.candle.timestamp, 1674485220);
    // determine profit loss
    assert_eq!(round(backtest_result.profit_loss, 2), 0.89);
    assert_eq!(round(backtest_result.profit_loss_percentage, 5), 0.00225);
    // determine peak
    assert_eq!(round(backtest_result.trade_peak_profit_loss_percentage, 5), 0.01454);
    // determine trough
    assert_eq!(round(backtest_result.trade_trough_profit_loss_percentage, 5), -0.00118);
  }
}
