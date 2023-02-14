use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use common::backtesting;
use common::backtesting::TradeBacktestResult;
use common::cache;
use common::candles;
use common::database;
use common::math;
use common::structs::*;
use common::trading;
use common::utilities;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use ordered_float::OrderedFloat;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

fn generate_backtest_context_combinations() -> Vec<BacktestContext> {
  let mut combinations = vec![];
  let min = dec!(0.001);
  let max = dec!(0.01);
  let step = dec!(0.0005);
  let profit_limit_percentages = utilities::build_decimal_range(min, max, step);
  let min = dec!(-0.01);
  let max = dec!(-0.001);
  let step = dec!(0.0005);
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
  // TODO
  return vec![
    TradeGenerationContext::default()
  ];
}

fn calculate_trade_result_performance(trade_results: &Vec<TradeBacktestResult>) -> (usize, f64) {
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  let mut num_trades = 0;
  for trade_result in trade_results {
    balance *= 1.0 + trade_result.profit_loss_percentage;
    num_trades += 1;
  }
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  return (num_trades, compounded_profit_loss_percentage);
}

fn print_progress(num_tested: usize, num_total: usize, start: Instant) {
  if num_tested % 100 == 0 {
    let elapsed_ms = start.elapsed().as_millis();
    let elapsed_sec = start.elapsed().as_secs();
    let rate_ms = num_tested as f64 / elapsed_ms as f64;
    let rate_sec = rate_ms * 1000.0;
    let num_left = num_total - num_tested;
    let eta_sec = num_left as f64 / rate_sec as f64;
    let percent = (num_tested as f64 / num_total as f64) * 100.0;
    log::info!("{}/{} {:.0}% elapsed {}s eta {:.0}s {:.0}/sec", num_tested, num_total, percent, elapsed_sec, eta_sec, rate_sec)
  }
}

fn main() {
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // config
  let args: Vec<String> = std::env::args().collect();
  let provider_name = args.get(1).unwrap();
  let symbol = args.get(2).unwrap();
  let resolution = args.get(3).unwrap();
  let dates_start = format!("{} 00:00:00", args.get(4).unwrap());
  let dates_end = format!("{} 15:59:59", args.get(5).unwrap());
  let dates = common::dates::build_list_of_dates(&dates_start, &dates_end);
  // open database + init database tables
  let database_filename = format!("./database-{}.db", provider_name);
  let connection = database::Database::new(&database_filename);
  connection.migrate("./schema/");
  // build candles cache map
  let candles_date_map = cache::build_candles_date_map(&connection, symbol, resolution, &dates);
  // build list of combinations
  let trade_generation_context_combinations = generate_trade_generation_context_combinations();
  let backtest_context_combinations = generate_backtest_context_combinations();
  let num_combinations = trade_generation_context_combinations.len() * backtest_context_combinations.len();
  log::info!("num_combinations = {}", num_combinations);
  // configure thread pool
  rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
  // backtest combinations
  let start = std::time::Instant::now();
  let combination_results: Vec<CombinationBacktestResult> = vec![];
  let combination_results = Arc::new(Mutex::new(combination_results));
  let candles = candles::get_candles_by_date_as_continuous_vec(&dates, &candles_date_map);
  trade_generation_context_combinations.par_iter().for_each(|trade_generation_context| {
    // build list of trades
    let trades = trading::generate_continuous_trades(&dates, &trade_generation_context, &candles);
    // backtest trades
    backtest_context_combinations.par_iter().for_each(|backtest_context| {
      let trade_results = backtesting::generate_trades_results(backtest_context, &trades, &candles);
      // summarize trade results
      let (num_trades, compounded_profit_loss_percentage) = calculate_trade_result_performance(&trade_results);
      if compounded_profit_loss_percentage >= 0.10 {
        log::info!("trade_generation_context = {:?} backtest_context = {:?} {:.2}", trade_generation_context, backtest_context, compounded_profit_loss_percentage);
      }
      let mut combination_results = combination_results.lock().unwrap();
      combination_results.push(CombinationBacktestResult {
        trade_generation_context: trade_generation_context.clone(),
        backtest_context: backtest_context.clone(),
        num_trades,
        compounded_profit_loss_percentage
      });
      print_progress(combination_results.len(), num_combinations, start);
    });
  });
  let mut combination_results = combination_results.lock().unwrap();
  /*let min_num_trades = combination_results.iter().map(|combination_result| combination_result.num_trades).min().unwrap();
  let max_num_trades = combination_results.iter().map(|combination_result| combination_result.num_trades).max().unwrap();
  let min_compounded_profit_loss_percentage = combination_results.iter().map(|combination_result| OrderedFloat(combination_result.compounded_profit_loss_percentage)).min().unwrap().into_inner();
  let max_compounded_profit_loss_percentage = combination_results.iter().map(|combination_result| OrderedFloat(combination_result.compounded_profit_loss_percentage)).max().unwrap().into_inner();*/
  combination_results.sort_by(|a, b| {
    /*let a_num_trades = math::normalize(a.num_trades as f64, min_num_trades as f64, max_num_trades as f64);
    let b_num_trades = math::normalize(b.num_trades as f64, min_num_trades as f64, max_num_trades as f64);
    let a_compounded_profit_loss_percentage = math::normalize(a.compounded_profit_loss_percentage, min_compounded_profit_loss_percentage, max_compounded_profit_loss_percentage);
    let b_compounded_profit_loss_percentage = math::normalize(b.compounded_profit_loss_percentage, min_compounded_profit_loss_percentage, max_compounded_profit_loss_percentage);
    let num_trades_weight = 0.10;
    let compounded_profit_loss_percentage_weight = 0.90;
    let a_score = num_trades_weight * (1.0 - a_num_trades) + compounded_profit_loss_percentage_weight * (a_compounded_profit_loss_percentage);
    let b_score = num_trades_weight * (1.0 - b_num_trades) + compounded_profit_loss_percentage_weight * (b_compounded_profit_loss_percentage);*/
    let a_score = a.compounded_profit_loss_percentage;
    let b_score = b.compounded_profit_loss_percentage;
    return b_score.partial_cmp(&a_score).unwrap();
  });
  let best_combination_result = &combination_results[0];
  log::info!("best_combination_result = {}", serde_json::to_string(&best_combination_result).unwrap());
  // flush trades to file?
  let trades = trading::generate_continuous_trades(&dates, &best_combination_result.trade_generation_context, &candles);
  let stringified_value = serde_json::to_string_pretty(&trades).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/trades.json")).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
  // flush backtest results to file
  let trade_results = backtesting::generate_trades_results(&best_combination_result.backtest_context, &trades, &candles);
  let stringified_value = serde_json::to_string_pretty(&trade_results).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/trade-results.json")).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
}
