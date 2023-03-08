fn main() {
  // load env vars
  dotenv::from_filename("./.env").ok().unwrap();
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // arguments
    let args: Vec<String> = std::env::args().collect();
    let provider_name = args.get(1).unwrap();
    let start_date = format!("{} 00:00:00", args.get(2).unwrap());
    let end_date = format!("{} 15:59:59", args.get(3).unwrap());
    // config
    let symbol = "SPY"; // TODO: do not hardcode?
    let resolution = "1"; // TODO: do not hardcode?
    // get dates
    let dates = common::dates::build_list_of_trading_dates(&start_date, &end_date);
    if dates.len() == 0 {
      panic!("no trading dates");
    }
    // loop dates
    for date in &dates {
      // formate date
      // TODO: regular vs extended?
      //let (from, to) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
      let (from, to) = common::market_session::get_extended_market_session_start_and_end_from_string(date);
      // get candles
      let result = providers::get_candles_by_provider_name(provider_name, symbol, resolution, from, to).await;
      if result.is_err() {
        log::error!("{:?}", result);
        continue;
      }
      let candles = result.unwrap();
      log::info!("got {} candles for {} -> {}", candles.len(), from, to);
      // TODO: got rid of sqlite, write to file?
    }
  });
}
