pub mod formatter;
pub mod helpers;
pub mod structs;

use self::structs::*;
use common::http_client;
use futures::StreamExt;
use log::info;
use std::collections::HashMap;

const KNOWN_ERRORS: &[&str] = &["timed out", "invalid response status: 502", "invalid response status: 503"];
const REQUEST_TIMEOUT_MS: u64 = 5000;
const RETRY_DELAY_MS: u64 = 1000;
const NUM_REQUEST_RETRIES: usize = 10;

pub struct Robinhood {
  http_client: reqwest::Client,
  uuid_map: HashMap<String, String>,
}

impl Robinhood {
  pub fn new() -> Robinhood {
    return Robinhood {
      http_client: reqwest::Client::new(),
      uuid_map: HashMap::from([(String::from("quote_id:SPY"), String::from("8f92e76f-1e0e-4478-8580-16a6ffcfaef5"))]),
    };
  }

  fn build_headers(&self, token: &str) -> Vec<(String, String)> {
    return vec![
      (String::from("origin"), String::from("https://robinhood.com")),
      (String::from("referer"), String::from("https://robinhood.com")),
      (
        String::from("user-agent"),
        String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36"),
      ),
      (String::from("authorization"), format!("Bearer {}", token)),
    ];
  }

  pub fn get_chain_id_from_symbol(&self, symbol: &str) -> &str {
    let key = format!("chain_id:{}", symbol);
    return self.uuid_map.get(&key).unwrap();
  }

  pub fn get_quote_id_from_symbol(&self, symbol: &str) -> &str {
    let key = format!("quote_id:{}", symbol);
    return self.uuid_map.get(&key).unwrap();
  }

  // TODO: use this instead of ROBINHOOD_API_TOKEN env var but it doesn't work for get_options_market_data_chunk (401s)
  pub async fn get_logged_out_access_token(&self) -> Result<String, String> {
    log::info!("get_logged_out_access_token");
    let request_headers = vec![
      (String::from("origin"), String::from("https://robinhood.com")),
      (String::from("referer"), String::from("https://robinhood.com")),
      (
        String::from("user-agent"),
        String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36"),
      ),
    ];
    let url = "https://robinhood.com/stocks/SPY/".to_string();
    // TODO: retries?
    let result = http_client::http_request_text(&self.http_client, "GET", &url, &request_headers, &None).await;
    if result.is_err() {
      return Err(format!("{:?}", result));
    }
    let (response_headers, _response_body) = result.unwrap();
    let set_cookie_header_value = response_headers.get("set-cookie").unwrap().to_str().unwrap();
    let parsed_cookie = cookie::Cookie::parse(set_cookie_header_value).unwrap();
    let parsed_cookie_value = parsed_cookie.value();
    return Ok(parsed_cookie_value.to_owned());
  }

  pub async fn get_quote(&self, token: &str, symbol: &str) -> Result<Quote, String> {
    log::info!("get_quote: symbol = {}", symbol);
    let quote_id = self.get_quote_id_from_symbol(symbol);
    let url = format!("https://api.robinhood.com/marketdata/quotes/{}/?bounds=trading", quote_id);
    return http_client::http_request_json_with_timeout_and_retries::<Quote>(
      &self.http_client,
      "GET",
      &url,
      &self.build_headers(token),
      &None,
      KNOWN_ERRORS,
      REQUEST_TIMEOUT_MS,
      RETRY_DELAY_MS,
      NUM_REQUEST_RETRIES,
    )
    .await;
  }

  pub async fn get_chain(&self, token: &str, symbol: &str) -> Result<Chain, String> {
    info!("get_chain: symbol = {}", symbol);
    let chain_id = self.get_chain_id_from_symbol(symbol);
    let url = format!("https://api.robinhood.com/options/chains/{}/", chain_id);
    return http_client::http_request_json_with_timeout_and_retries::<Chain>(
      &self.http_client,
      "GET",
      &url,
      &self.build_headers(token),
      &None,
      KNOWN_ERRORS,
      REQUEST_TIMEOUT_MS,
      RETRY_DELAY_MS,
      NUM_REQUEST_RETRIES,
    )
    .await;
  }

  #[async_recursion::async_recursion]
  pub async fn get_options_instruments(
    &self,
    token: &str,
    symbol: &str,
    expiration_date: &str,
    state: &str,
    r#type: &str,
    cursor: Option<String>,
  ) -> Result<Vec<OptionInstrument>, String> {
    info!(
      "get_options_instruments: symbol = {} expiration_date = {} state = {} type = {} cursor = {:?}",
      symbol, expiration_date, state, r#type, cursor
    );
    let chain_id = self.get_chain_id_from_symbol(symbol);
    let mut base_url = url::Url::parse("https://api.robinhood.com/options/instruments/").unwrap();
    base_url.query_pairs_mut().append_pair("chain_id", chain_id);
    base_url.query_pairs_mut().append_pair("expiration_dates", expiration_date);
    base_url.query_pairs_mut().append_pair("state", state);
    base_url.query_pairs_mut().append_pair("type", r#type);
    if cursor.is_some() {
      base_url.query_pairs_mut().append_pair("cursor", &cursor.unwrap());
    }
    let stringified_url = base_url.as_str().to_string();
    let response = http_client::http_request_json_with_timeout_and_retries::<OptionsInstrumentsResults>(
      &self.http_client,
      "GET",
      &stringified_url,
      &self.build_headers(token),
      &None,
      KNOWN_ERRORS,
      REQUEST_TIMEOUT_MS,
      RETRY_DELAY_MS,
      NUM_REQUEST_RETRIES,
    )
    .await;
    if response.is_err() {
      return Err(format!("{:?}", response));
    }
    let mut response = response.unwrap();
    let mut results = vec![];
    results.append(&mut response.results);
    if response.next.is_some() {
      let next_url = response.next.unwrap();
      let parsed_next_url = url::Url::parse(&next_url).unwrap();
      let parsed_next_url_query_parameters: std::collections::HashMap<_, _> = parsed_next_url.query_pairs().into_owned().collect();
      let next_cursor = parsed_next_url_query_parameters.get("cursor").unwrap().to_string();
      let next_page = self
        .get_options_instruments(token, chain_id, expiration_date, state, r#type, Some(next_cursor))
        .await;
      if next_page.is_err() {
        return Err(format!("{:?}", next_page));
      }
      let mut next_page = next_page.unwrap();
      results.append(&mut next_page);
    }
    return Ok(results);
  }

  pub async fn get_options_market_data_chunk(&self, token: &str, instrument_ids_chunk: &[String]) -> Result<Vec<OptionMarketData>, String> {
    info!("get_options_market_data_chunk");
    let joined_instrument_ids = instrument_ids_chunk.join(",");
    let mut base_url = url::Url::parse("https://api.robinhood.com/marketdata/options/").unwrap();
    base_url.query_pairs_mut().append_pair("ids", &joined_instrument_ids);
    let stringified_url = base_url.as_str().to_string();
    let response = http_client::http_request_json_with_timeout_and_retries::<OptionsMarketDataResult>(
      &self.http_client,
      "GET",
      &stringified_url,
      &self.build_headers(token),
      &None,
      KNOWN_ERRORS,
      REQUEST_TIMEOUT_MS,
      RETRY_DELAY_MS,
      NUM_REQUEST_RETRIES,
    )
    .await;
    if response.is_err() {
      return Err(format!("{:?}", response));
    }
    let response = response.unwrap();
    let chunk_results: Vec<OptionMarketData> = response
      .results
      .into_iter()
      .filter(|result| {
        return result.is_some();
      })
      .map(|result| {
        return result.unwrap();
      })
      .collect();
    return Ok(chunk_results);
  }

  pub async fn get_options_market_data(&self, token: &str, instrument_ids: &Vec<String>) -> Result<Vec<OptionMarketData>, String> {
    info!("get_options_market_data instrument_ids.len() = {}", instrument_ids.len());
    let chunk_size = 128;
    let instrument_ids_chunks: Vec<Vec<String>> = instrument_ids.chunks(chunk_size).map(|s| s.to_vec()).collect();
    let futures = instrument_ids_chunks.iter().map(|instrument_ids_chunk| {
      return self.get_options_market_data_chunk(token, instrument_ids_chunk);
    });
    let concurrency = 4;
    let results = futures::stream::iter(futures).buffer_unordered(concurrency).collect::<Vec<_>>().await;
    for result in &results {
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
    }
    let flattened_results: Vec<OptionMarketData> = results.into_iter().map(|result| result.unwrap()).flatten().collect();
    return Ok(flattened_results);
  }

  pub async fn get_options_series_by_type(
    &self,
    token: &str,
    symbol: &str,
    expiration_date: &str,
    r#type: &str,
    min_strike: f64,
    max_strike: f64,
  ) -> Result<OptionSeries, String> {
    info!(
      "get_options_series_by_type: symbol = {} expiration_date = {} type = {}",
      symbol, expiration_date, r#type
    );
    let chain_id = self.get_chain_id_from_symbol(symbol);
    let options = self.get_options_instruments(token, chain_id, expiration_date, "active", r#type, None).await;
    if options.is_err() {
      return Err(format!("{:?}", options));
    }
    let options = options.unwrap();
    let filtered_options: Vec<OptionInstrument> = options
      .into_iter()
      .filter(|option| {
        let strike_price = option.strike_price.parse::<f64>().unwrap();
        return strike_price >= min_strike && strike_price <= max_strike;
      })
      .collect();
    // option quotes
    let option_ids: Vec<String> = filtered_options.iter().map(|option| option.id.to_owned()).collect();
    let options_quotes = self.get_options_market_data(token, &option_ids).await;
    if options_quotes.is_err() {
      return Err(format!("{:?}", options_quotes));
    }
    let options_quotes = options_quotes.unwrap();
    // return
    return Ok(OptionSeries {
      options: filtered_options,
      options_quotes,
    });
  }

  pub async fn get_options_series(&self, token: &str, symbol: &str, expiration_date: &str, min_strike: f64, max_strike: f64) -> Result<OptionSeries, String> {
    info!("get_options_series: symbol = {} expiration_date = {}", symbol, expiration_date);
    let chain_id = self.get_chain_id_from_symbol(symbol);
    let call_future = self.get_options_series_by_type(token, chain_id, expiration_date, "call", min_strike, max_strike);
    let put_future = self.get_options_series_by_type(token, chain_id, expiration_date, "put", min_strike, max_strike);
    let futures = vec![call_future, put_future];
    let results = futures::future::join_all(futures).await;
    let call_option_series = results.get(0).unwrap().to_owned();
    let put_option_series = results.get(1).unwrap().to_owned();
    if call_option_series.is_err() {
      return Err(format!("{:?}", call_option_series));
    }
    if put_option_series.is_err() {
      return Err(format!("{:?}", put_option_series));
    }
    let call_option_series = call_option_series.unwrap();
    let put_option_series = put_option_series.unwrap();
    let mut options = vec![];
    options.extend(call_option_series.options.to_owned());
    options.extend(put_option_series.options.to_owned());
    let mut options_quotes = vec![];
    options_quotes.extend(call_option_series.options_quotes);
    options_quotes.extend(put_option_series.options_quotes);
    return Ok(OptionSeries { options, options_quotes });
  }
}
