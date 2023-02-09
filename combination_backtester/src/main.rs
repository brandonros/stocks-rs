use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use common::backtesting;
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

fn generate_combinations() -> Vec<(TradeGenerationContext, BacktestContext)> {
  let mut combinations = vec![];
  let min = dec!(-0.01);
  let max = dec!(0.01);
  let step = dec!(0.00005);
  let divergence_thresholds = utilities::build_decimal_range(min, max, step);
  let min = dec!(0.001);
  let max = dec!(0.005);
  let step = dec!(0.0005);
  let profit_limit_percentages = utilities::build_decimal_range(min, max, step);
  let min = dec!(-0.005);
  let max = dec!(-0.001);
  let step = dec!(0.0005);
  let stop_loss_percentages = utilities::build_decimal_range(min, max, step);
  for divergence_threshold in &divergence_thresholds {
    for profit_limit_percentage in &profit_limit_percentages {
      for stop_loss_percentage in &stop_loss_percentages {
        let trade_generation_context = TradeGenerationContext {
          vwap_std_dev_multiplier: 1.5,
          warmup_periods: 10,
          sma_periods: 10,
          divergence_threshold: divergence_threshold.to_f64().unwrap()
        };
        let backtest_context = BacktestContext {
          slippage_percentage: 0.000125,
          profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
          stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
        };
        combinations.push((trade_generation_context, backtest_context));
      }
    }
  }
  return combinations;
}

fn backtest_combination(
  dates: &Vec<String>,
  strategy_name: &String,
  candles_date_map: &HashMap<String, Vec<Arc<Candle>>>,
  trade_generation_context: &TradeGenerationContext,
  backtest_context: &BacktestContext,
) -> f64 {
  // build list of trades
  let dates_trades_map = trading::generate_dates_trades_map(&dates, &trade_generation_context, strategy_name, &candles_date_map);
  // backtest trades
  let dates_trades_results_map = backtesting::generate_dates_trades_results_map(&dates, &backtest_context, &candles_date_map, &dates_trades_map);
  // summarize trade results
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
  let combinations = generate_combinations();
  let num_combinations = combinations.len();
  log::info!("num_combinations = {}", num_combinations);
  let start = std::time::Instant::now();
  let combination_results: Vec<f64> = vec![];
  let combination_results = Arc::new(Mutex::new(combination_results));
  combinations.par_iter().for_each(|combination| {
    let trade_generation_context = &combination.0;
    let backtest_context = &combination.1;
    let compounded_profit_loss_percentage = backtest_combination(&dates, strategy_name, &candles_date_map, &trade_generation_context, &backtest_context);
    let mut combination_results = combination_results.lock().unwrap();
    combination_results.push(compounded_profit_loss_percentage);
    let num_tested = combination_results.len();
    if num_tested % 10 == 0 {
      let elapsed_ms = start.elapsed().as_millis();
      let elapsed_sec = start.elapsed().as_secs();
      let rate_sec = num_tested as f64 / elapsed_sec as f64;
      let num_left = num_combinations - num_tested;
      let eta_sec = num_left as f64 / rate_sec as f64;
      log::info!("{}/{} elapsed {}s eta {}s {}/sec", num_tested, num_combinations, elapsed_sec, eta_sec, rate_sec)
    }
  });
  let mut combination_results = combination_results.lock().unwrap();
  combination_results.sort_by(|a, b| {
    return b.partial_cmp(&a).unwrap();
  });
  let best_combination_result = &combination_results[0];
  log::info!("best_combination_result = {:.2}", best_combination_result);
}
