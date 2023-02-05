use providers::Provider;
use strategies::Strategy;

mod backtesting;
mod structs;

fn main() {
  // load env vars
  dotenv::from_filename("./.env").ok().unwrap();
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    let args: Vec<String> = std::env::args().collect();
    let provider_name = args.get(1).unwrap();
    let strategy_name = args.get(2).unwrap();
    let symbol = args.get(3).unwrap();
    let resolution = args.get(4).unwrap();
    let start_date = format!("{} 00:00:00", args.get(5).unwrap());
    let end_date = format!("{} 00:00:00", args.get(6).unwrap());
    let provider: Provider = provider_name.parse().unwrap();
    let strategy: Strategy = strategy_name.parse().unwrap();
    let dates = common::dates::build_list_of_dates(&start_date, &end_date);
    let dates: Vec<&str> = dates.iter().map(|date| return date.as_str()).collect();
    backtesting::backtest(symbol, resolution, &provider, &strategy, &dates).await;
  });
}
