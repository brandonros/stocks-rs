use providers::Provider;
use strategies::Strategy;

mod backtesting;
mod computing;
mod dates;
mod market_session;
mod math;
mod signals;
mod structs;

fn main() {
  // load env vars
  dotenv::from_filename("./.env").ok().unwrap();
  // logger
  simple_logger::SimpleLogger::new().env().init().unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).unwrap();
    if command == "backtest" {
      let provider_name = args.get(2).unwrap();
      let strategy_name = args.get(3).unwrap();
      let symbol = args.get(4).unwrap();
      let resolution = args.get(5).unwrap();
      let start_date = args.get(6).unwrap();
      let end_date = args.get(7).unwrap();
      let provider: Provider = provider_name.parse().unwrap();
      let strategy: Strategy = strategy_name.parse().unwrap();
      let dates = dates::build_list_of_dates(&start_date, &end_date);
      let dates: Vec<&str> = dates.iter().map(|date| return date.as_str()).collect();
      backtesting::backtest(symbol, resolution, &provider, &strategy, &dates).await;
    } else if command == "compute" {
      let provider_name = args.get(2).unwrap();
      let strategy_name = args.get(3).unwrap();
      let symbol = args.get(4).unwrap();
      let resolution = args.get(5).unwrap();
      let date = args.get(6).unwrap();
      let provider: Provider = provider_name.parse().unwrap();
      let strategy: Strategy = strategy_name.parse().unwrap();
      computing::compute(symbol, resolution, &provider, &strategy, &date).await;
    } else {
      panic!("unknown command");
    }
  });
}
