use chrono::DateTime;
use chrono_tz::Tz;
use common::http_client;
use common::structs::*;

use crate::structs::*;

pub async fn get_candles(symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>, String> {
  let from_timestamp = from.timestamp_millis();
  let to_timestamp = to.timestamp_millis();
  log::info!("get_candles symbol = {} resolution = {} from = {} to = {}", symbol, resolution, from, to);
  let http_client = reqwest::Client::new();
  let polygon_api_token = std::env::var("POLYGON_API_TOKEN").unwrap();
  let mut request_url = url::Url::parse(&format!(
    "https://api.polygon.io/v2/aggs/ticker/{symbol}/range/{resolution}/minute/{from_timestamp}/{to_timestamp}"
  ))
  .unwrap();
  request_url.query_pairs_mut().append_pair("adjusted", "true");
  request_url.query_pairs_mut().append_pair("sort", "asc");
  request_url.query_pairs_mut().append_pair("limit", "500");
  request_url.query_pairs_mut().append_pair("apiKey", &polygon_api_token);
  let request_url = request_url.as_str().to_string();
  let request_headers = vec![];
  let result = http_client::http_request_json::<PolygonResponseRoot>(&http_client, "GET", &request_url, &request_headers, &None).await;
  if result.is_err() {
    return Err(result.err().unwrap());
  }
  let response_body = result.unwrap();
  let mut candles = vec![];
  for result in response_body.results {
    let timestamp = result.t / 1000;
    let open = result.o;
    let high = result.h;
    let low = result.l;
    let close = result.c;
    let volume = result.v as i64;
    candles.push(Candle {
      symbol: symbol.to_string(),
      resolution: resolution.to_string(),
      timestamp,
      open,
      high,
      low,
      close,
      volume,
    });
  }
  return Ok(candles);
}
