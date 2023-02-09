use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use common::cache;
use common::candles;
use common::database;
use common::database::Database;
use common::market_session;
use common::structs::*;
use common::trading;

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
  // build list of trades
  let trade_generation_context = TradeGenerationContext {
    vwap_std_dev_multiplier: 1.5,
    warmup_periods: 10,
    sma_periods: 10,
    divergence_threshold: 0.00025,
  };
  let dates_trades_map = trading::generate_dates_trades_map(&dates, &trade_generation_context, strategy_name, &candles_date_map);
  // flush trades to file?
  let stringified_value = serde_json::to_string_pretty(&dates_trades_map).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/{}-trades.json", strategy_name)).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
}
