use super::structs::*;
use super::ThinkOrSwim;

use chrono::Utc;
use futures::StreamExt;
use log::info;

pub async fn get_symbol_series_name_snapshots(
  token: String,
  symbol: String,
  series_name: String,
  expiration_date: String,
  strike_price_threshold: f64,
) -> Result<(Quote, OptionChain, Vec<OptionChainQuote>), String> {
  info!(
    "get_symbol_series_name_snapshots: symbol = {} series_name = {} expiration_date = {}",
    symbol, series_name, expiration_date
  );
  // init tos
  let tos = ThinkOrSwim::new();
  // connect
  tos.connect().await;
  // heartbeat
  tos.init_heartbeat().await;
  // login
  let login_response = tos.login(token.to_owned()).await;
  if login_response.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", login_response));
  }
  // get underlying quote
  let quote = tos.get_quote(symbol.to_owned()).await;
  if quote.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", quote));
  }
  let quote = quote.unwrap();
  // get option chain + quotes
  let option_chain = tos.get_option_chain(symbol.to_owned(), series_name.to_owned()).await;
  if option_chain.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", option_chain));
  }
  let option_chain = option_chain.unwrap();
  let min_strike = quote.values.MARK.unwrap() * (1.0 - strike_price_threshold);
  let max_strike = quote.values.MARK.unwrap() * (1.0 + strike_price_threshold);
  let option_chain_quotes = tos
    .get_option_chain_quotes(symbol.to_owned(), series_name.to_owned(), min_strike as usize, max_strike as usize)
    .await;
  if option_chain_quotes.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", option_chain_quotes));
  }
  let option_chain_quotes = option_chain_quotes.unwrap();
  // shutdown tos
  tos.shutdown.shutdown();
  // return
  return Ok((quote, option_chain, option_chain_quotes));
}

/*
do not use expiration, use lastTradeDate property instead
example:
{
  "underlying": "SPY",
  "name": "16 DEC 22 100",
  "spc": 100.0,
  "multiplier": 100.0,
  "expirationStyle": "REGULAR",
  "isEuropean": false,
  "expiration": "2022-12-17T12:00:00Z",
  "lastTradeDate": "2022-12-16T21:00:00Z",
  "settlementType": "PM"
}
*/
pub async fn get_series_names_and_expiration_dates(token: String, symbol: String) -> Result<Vec<(String, String)>, String> {
  // init tos
  let tos = ThinkOrSwim::new();
  // connect
  tos.connect().await;
  // heartbeat
  tos.init_heartbeat().await;
  // login
  let login_response = tos.login(token.to_owned()).await;
  if login_response.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", login_response));
  }
  // get option series + quotes
  let option_series = tos.get_option_series(symbol.to_owned()).await;
  if option_series.is_err() {
    tos.shutdown.shutdown();
    return Err(format!("{:?}", option_series));
  }
  let option_series = option_series.unwrap();
  // shutdown tos
  tos.shutdown.shutdown();
  // return
  let formatted_option_series = option_series
    .iter()
    .map(|option_series| {
      return (option_series.name.to_owned(), option_series.lastTradeDate.to_owned());
    })
    .collect();
  return Ok(formatted_option_series);
}

pub async fn scrape_symbol_options_chain(
  token: String,
  symbol: String,
  strike_price_threshold: f64,
  days_to_expiration_threshold: f64,
) -> Result<Vec<(Quote, OptionChain, Vec<OptionChainQuote>)>, String> {
  let now = Utc::now().naive_utc();
  let series_names_and_expiration_dates = get_series_names_and_expiration_dates(token.to_owned(), symbol.to_owned()).await;
  if series_names_and_expiration_dates.is_err() {
    return Err(format!("{:?}", series_names_and_expiration_dates));
  }
  let series_names_and_expiration_dates = series_names_and_expiration_dates.unwrap();
  let filtered_series_names_and_expiration_dates: Vec<(String, String)> = series_names_and_expiration_dates
    .into_iter()
    .filter(|(_series_name, expiration_date)| {
      let parsed_expiration_date = chrono::NaiveDateTime::parse_from_str(expiration_date, "%Y-%m-%dT%H:%M:%SZ").unwrap();
      let diff = parsed_expiration_date.signed_duration_since(now);
      let seconds_per_day = 86400.0;
      let days_to_expiration = diff.num_seconds() as f64 / seconds_per_day;
      return days_to_expiration <= days_to_expiration_threshold;
    })
    .collect();
  info!("fetching {} series", filtered_series_names_and_expiration_dates.len());
  let futures = filtered_series_names_and_expiration_dates.into_iter().map(|(series_name, expiration_date)| {
    return get_symbol_series_name_snapshots(token.to_owned(), symbol.to_owned(), series_name, expiration_date, strike_price_threshold);
  });
  let concurrency = 16;
  let results = futures::stream::iter(futures).buffer_unordered(concurrency).collect::<Vec<_>>().await;
  for result in &results {
    if result.is_err() {
      return Err(format!("{:?}", result));
    }
  }
  let flattened_results: Vec<(Quote, OptionChain, Vec<OptionChainQuote>)> = results.into_iter().map(|result| result.unwrap()).collect();
  return Ok(flattened_results);
}
