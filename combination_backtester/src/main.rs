use std::collections::HashMap;
use std::sync::Arc;

use common::backtesting;
use common::cache;
use common::database;
use common::math;
use common::structs::*;
use common::trading;

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
  let trade_generation_context = TradeGenerationContext {
    vwap_std_dev_multiplier: 1.5,
    warmup_periods: 10,
    sma_periods: 10,
    divergence_threshold: 0.00025,
  };
  let backtest_context = BacktestContext {
    slippage_percentage: 0.000125,
    profit_limit_percentage: 0.004,
    stop_loss_percentage: -0.002,
  };
  let combinations = vec![(trade_generation_context, backtest_context)];
  let mut combination_results: Vec<f64> = combinations
    .iter()
    .map(|combination| {
      let trade_generation_context = &combination.0;
      let backtest_context = &combination.1;
      return backtest_combination(&dates, strategy_name, &candles_date_map, &trade_generation_context, &backtest_context);
    })
    .collect();
  combination_results.sort_by(|a, b| {
    return b.partial_cmp(&a).unwrap();
  });
  let best_combination_result = &combination_results[0];
  log::info!("best_combination_result = {:.2}", best_combination_result);
}
