fn main() {
  // load env vars
  dotenv::from_filename("./.env").ok().unwrap();
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let provider_name = "polygon";
    let symbol = "SPY";
    let resolution = "1";
    // open database
    let connection = common::database::Database::new(&format!("./database-{}.db", provider_name));
    // init database tables
    connection.migrate("./schema/");
    // get dates
    let args: Vec<String> = std::env::args().collect();
    let start_date = format!("{} 00:00:00", args.get(1).unwrap());
    let end_date = format!("{} 00:00:00", args.get(2).unwrap());
    let dates = common::dates::build_list_of_dates(&start_date, &end_date);
    // loop dates
    for date in &dates {
      // formate date
      let (from, to) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
      // get candles
      let result = providers::get_candles_by_provider_name(provider_name, symbol, resolution, from, to).await;
      if result.is_err() {
        panic!("TODO");
      }
      let candles = result.unwrap();
      log::info!("got {} candles for {}", candles.len(), date);
      // insert into database
      for candle in &candles {
        let result = connection.insert(candle);
        if result.is_err() {
          panic!("TODO");
        }
      }
    }
  });
}
