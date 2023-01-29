use crate::{
  database, market_session,
  providers::{self, Provider},
};

pub async fn scrape(symbol: &str, resolution: &str, provider: &Provider, date: &str) {
  // connect to databse
  let connection = database::get_database_connection(&format!("{:?}", provider));
  database::init_tables(&connection);
  // build timestamp range from date
  let (from, to) = market_session::get_regular_market_start_end_from_string(date);
  log::info!("date = {} resolution = {}", date, resolution);
  // call api
  let result = match provider {
    Provider::Finnhub => providers::finnhub::get_candles(symbol, resolution, from, to).await,
    Provider::YahooFinance => providers::yahoo_finance::get_candles(symbol, resolution, from, to).await,
    Provider::Polygon => providers::polygon::get_candles(symbol, resolution, from, to).await,
  };
  // watch out for weird no_data error
  if result.is_ok() {
    let candles = result.unwrap();
    log::info!("date = {} num_candles = {}", date, candles.len());
    database::insert_candles_to_database(&connection, symbol, &resolution, &candles);
  } else {
    panic!("{:?}", result.err().unwrap());
  }
  // sleep due to finnhub API limit
  tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
}
