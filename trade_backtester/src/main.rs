use std::collections::HashMap;
use std::io::Write;

use common::backtesting;
use common::cache;
use common::database;
use common::math;
use common::structs::*;

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
  // read list of trades
  let stringified_trades = std::fs::read_to_string(format!("/tmp/{}-trades.json", strategy_name)).unwrap();
  let dates_trades_map: HashMap<String, Vec<Trade>> = serde_json::from_str(&stringified_trades).unwrap();
  // backtest trades
  let backtest_context = BacktestContext {
    slippage_percentage: 0.000125,
    profit_limit_percentage: 0.004,
    stop_loss_percentage: -0.002,
  };
  let dates_trades_results_map = backtesting::generate_dates_trades_results_map(&dates, &backtest_context, &candles_date_map, &dates_trades_map);
  // write to file
  let stringified_value = serde_json::to_string_pretty(&dates_trades_results_map).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/{}-trade-results.json", strategy_name)).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
  // print result
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  for date in &dates {
    let date_trade_results = dates_trades_results_map.get(date).unwrap();
    for date_trade_result in date_trade_results {
      balance *= 1.0 + date_trade_result.profit_loss_percentage;
    }
  }
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  log::info!("compounded_profit_loss_percentage = {:.2}", compounded_profit_loss_percentage);
}
