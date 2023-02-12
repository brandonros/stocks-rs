use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use common::backtesting;
use common::backtesting::TradeBacktestResult;
use common::cache;
use common::database;
use common::math;
use common::structs::*;
use common::trading;
use common::utilities;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

fn generate_backtest_context_combinations() -> Vec<BacktestContext> {
  let mut combinations = vec![];
  let min = dec!(0.001);
  let max = dec!(0.01);
  let step = dec!(0.001);
  let profit_limit_percentages = utilities::build_decimal_range(min, max, step);
  let min = dec!(-0.01);
  let max = dec!(-0.001);
  let step = dec!(0.001);
  let stop_loss_percentages = utilities::build_decimal_range(min, max, step);
  for profit_limit_percentage in &profit_limit_percentages {
    for stop_loss_percentage in &stop_loss_percentages {
      let backtest_context = BacktestContext {
        slippage_percentage: 0.000125,
        profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
        stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
      };
      combinations.push(backtest_context);
    }
  }
  return combinations;
}

fn generate_trade_generation_context_combinations() -> Vec<TradeGenerationContext> {
  let mut combinations = vec![];
  let min = dec!(0.0001);
  let max = dec!(0.01);
  let step = dec!(0.0001);
  let divergence_thresholds = utilities::build_decimal_range(min, max, step);
  for divergence_threshold in &divergence_thresholds {
    let trade_generation_context = TradeGenerationContext {
      vwap_std_dev_multiplier: 1.5,
      warmup_periods: 10,
      sma_periods: 10,
      divergence_threshold: divergence_threshold.to_f64().unwrap()
    };
    combinations.push(trade_generation_context);
  }
  return combinations;
}

fn calculate_trade_result_performance(dates: &Vec<String>, dates_trades_results_map: &HashMap<String, Vec<TradeBacktestResult>>) -> f64 {
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  for date in dates {
    let date_trade_results = dates_trades_results_map.get(date).unwrap();
    for date_trade_result in date_trade_results {
      balance *= 1.0 + date_trade_result.profit_loss_percentage;
    }
  }
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  return compounded_profit_loss_percentage;
}

fn print_progress(num_tested: usize, num_total: usize, start: Instant) {
  if num_tested % 100 == 0 {
    let elapsed_sec = start.elapsed().as_secs();
    let rate_sec = num_tested as f64 / elapsed_sec as f64;
    let num_left = num_total - num_tested;
    let eta_sec = num_left as f64 / rate_sec as f64;
    log::info!("{}/{} elapsed {}s eta {:.0}s {:.2}/sec", num_tested, num_total, elapsed_sec, eta_sec, rate_sec)
  }
}

fn main() {
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // config
  let args: Vec<String> = std::env::args().collect();
  let provider_name = args.get(1).unwrap();
  let strategy_name = args.get(2).unwrap();
  let symbol = args.get(3).unwrap();
  let resolution = args.get(4).unwrap();
  let dates_start = format!("{} 00:00:00", args.get(5).unwrap());
  let dates_end = format!("{} 00:00:00", args.get(6).unwrap());
  let dates = common::dates::build_list_of_dates(&dates_start, &dates_end);
  // open database + init database tables
  let database_filename = format!("./database-{}.db", provider_name);
  let connection = database::Database::new(&database_filename);
  connection.migrate("./schema/");
  // build candles cache map
  let candles_date_map = cache::build_candles_date_map(&connection, symbol, resolution, &dates);
  // build list of combinations
  //let trade_generation_context_combinations = generate_trade_generation_context_combinations();
  //let backtest_context_combinations = generate_backtest_context_combinations();
  let backtest_context_combinations = vec![
    BacktestContext {
      slippage_percentage: 0.000125,
      stop_loss_percentage: -0.004,
      profit_limit_percentage: 0.004
    }
  ];
  let trade_generation_context_combinations = vec![
    TradeGenerationContext {
      vwap_std_dev_multiplier: 1.5,
      divergence_threshold: 0.002,
      sma_periods: 10,
      warmup_periods: 10
    }
  ];
  let num_combinations = trade_generation_context_combinations.len() * backtest_context_combinations.len();
  log::info!("num_combinations = {}", num_combinations);
  // configure thread pool
  rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
  // backtest combinations
  let start = std::time::Instant::now();
  let combination_results: Vec<CombinationBacktestResult> = vec![];
  let combination_results = Arc::new(Mutex::new(combination_results));
  trade_generation_context_combinations.par_iter().for_each(|trade_generation_context| {
    // build list of trades
    let dates_trades_map = trading::generate_dates_trades_map(&dates, &trade_generation_context, strategy_name, &candles_date_map);
    // backtest trades
    backtest_context_combinations.par_iter().for_each(|backtest_context| {
      let dates_trades_results_map = backtesting::generate_dates_trades_results_map(&dates, &backtest_context, &candles_date_map, &dates_trades_map);
      // summarize trade results
      let compounded_profit_loss_percentage = calculate_trade_result_performance(&dates, &dates_trades_results_map);
      if compounded_profit_loss_percentage >= 0.05 {
        log::info!("trade_generation_context = {:?} backtest_context = {:?} {:.2}", trade_generation_context, backtest_context, compounded_profit_loss_percentage);
      }
      let mut combination_results = combination_results.lock().unwrap();
      combination_results.push(CombinationBacktestResult {
        trade_generation_context: trade_generation_context.clone(),
        backtest_context: backtest_context.clone(),
        compounded_profit_loss_percentage
      });
      print_progress(combination_results.len(), num_combinations, start);
    });
  });
  let mut combination_results = combination_results.lock().unwrap();
  combination_results.sort_by(|a, b| {
    return b.compounded_profit_loss_percentage.partial_cmp(&a.compounded_profit_loss_percentage).unwrap();
  });
  let best_combination_result = &combination_results[0];
  log::info!("best_combination_result = {:?}", best_combination_result);
}
