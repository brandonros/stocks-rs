use std::io::Write;

use common::cache;
use common::database;
use common::structs::*;
use common::trading;

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
  // build list of trades
  let trade_generation_context = TradeGenerationContext::default();
  let dates_trades_map = trading::generate_dates_trades_map(&dates, &trade_generation_context, &candles_date_map);
  // flush trades to file?
  let stringified_value = serde_json::to_string_pretty(&dates_trades_map).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/trades.json")).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
}
