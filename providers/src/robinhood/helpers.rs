use chrono::Utc;
use futures::StreamExt;

use super::{structs::*, Robinhood};

pub async fn get_expiration_dates(token: &str, symbol: &str) -> Result<Vec<String>, String> {
  let robinhood = Robinhood::new();
  let chain_id = robinhood.get_chain_id_from_symbol(symbol);
  let chain = robinhood.get_chain(token, chain_id).await;
  if chain.is_err() {
    return Err(format!("{:?}", chain));
  }
  let chain = chain.unwrap();
  let expiration_dates = chain.expiration_dates;
  return Ok(expiration_dates);
}

pub async fn scrape_symbol_expiration_date_options_chain(
  token: &str,
  symbol: &str,
  expiration_date: &str,
  strike_price_threshold: f64,
) -> Result<OptionSeries, String> {
  let robinhood = Robinhood::new();
  let quote = robinhood.get_quote(token, symbol).await;
  if quote.is_err() {
    return Err(format!("{:?}", quote));
  }
  let quote = quote.unwrap();
  let quote_ask_price = quote.ask_price.parse::<f64>().unwrap();
  let quote_bid_price = quote.bid_price.parse::<f64>().unwrap();
  let mark_price = (quote_bid_price + quote_ask_price) / 2.0;
  let min_strike = mark_price * (1.0 - strike_price_threshold);
  let max_strike = mark_price * (1.0 + strike_price_threshold);
  return robinhood.get_options_series(token, symbol, expiration_date, min_strike, max_strike).await;
}

pub async fn scrape_symbol_options_chain(
  token: &str,
  symbol: &str,
  strike_price_threshold: f64,
  days_to_expiration_threshold: f64,
) -> Result<Vec<OptionSeries>, String> {
  let now = Utc::now().naive_utc();
  let expiration_dates = get_expiration_dates(token, symbol).await;
  if expiration_dates.is_err() {
    return Err(format!("{:?}", expiration_dates));
  }
  let expiration_dates = expiration_dates.unwrap();
  let filtered_expiration_dates: Vec<String> = expiration_dates
    .into_iter()
    .filter(|expiration_date| {
      let parsed_expiration_date = chrono::NaiveDateTime::parse_from_str(&format!("{} 21:00:00", expiration_date), "%Y-%m-%d %H:%M:%S").unwrap();
      let diff = parsed_expiration_date.signed_duration_since(now);
      let seconds_per_day = 86400.0;
      let days_to_expiration = diff.num_seconds() as f64 / seconds_per_day;
      return days_to_expiration <= days_to_expiration_threshold;
    })
    .collect();
  let futures = filtered_expiration_dates.iter().map(|expiration_date| {
    return scrape_symbol_expiration_date_options_chain(token, symbol, expiration_date, strike_price_threshold);
  });
  let concurrency = 16;
  let results = futures::stream::iter(futures).buffer_unordered(concurrency).collect::<Vec<_>>().await;
  for result in &results {
    if result.is_err() {
      return Err(format!("{:?}", result));
    }
  }
  let flattened_results: Vec<OptionSeries> = results.into_iter().map(|result| result.unwrap()).collect();
  return Ok(flattened_results);
}
