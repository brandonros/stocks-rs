pub mod structs;

use anyhow::Result;
use chrono::DateTime;
use chrono_tz::Tz;
use common::file;
use common::http_client;
use common::structs::*;
use structs::*;

pub struct Polygon {
  http_client: reqwest::Client,
}

impl Polygon {
  pub fn new() -> Polygon {
    return Polygon {
      http_client: reqwest::Client::new(),
    };
  }

  pub fn get_cached_candles(&self, symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>> {
    let from_timestamp = from.timestamp_millis();
    let to_timestamp = to.timestamp_millis();
    log::info!("get_candles symbol = {} resolution = {} from = {} to = {}", symbol, resolution, from, to);
    let filename = format!("./data/polygon-{symbol}-{resolution}-{from_timestamp}-{to_timestamp}.json");
    let parsed_response_body = file::sync_read_json_from_file::<PolygonResponseRoot>(&filename);
    let mut candles = vec![];
    for result in parsed_response_body.results {
      let timestamp = result.t / 1000;
      let open = result.o;
      let high = result.h;
      let low = result.l;
      let close = result.c;
      let volume = result.v as i64;
      candles.push(Candle {
        //symbol: symbol.to_string(),
        //resolution: resolution.to_string(),
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

  pub async fn get_candles(&self, symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>> {
    let from_timestamp = from.timestamp_millis();
    let to_timestamp = to.timestamp_millis();
    log::info!("get_candles symbol = {} resolution = {} from = {} to = {}", symbol, resolution, from, to);
    let polygon_api_token = std::env::var("POLYGON_API_TOKEN").unwrap();
    let mut request_url = url::Url::parse(&format!(
      "https://api.polygon.io/v2/aggs/ticker/{symbol}/range/{resolution}/minute/{from_timestamp}/{to_timestamp}"
    ))
    .unwrap();
    request_url.query_pairs_mut().append_pair("adjusted", "true");
    request_url.query_pairs_mut().append_pair("sort", "asc");
    request_url.query_pairs_mut().append_pair("limit", "1000");
    request_url.query_pairs_mut().append_pair("apiKey", &polygon_api_token);
    let request_url = request_url.as_str().to_string();
    let request_headers = vec![];
    let (_response_headers, response_body) = http_client::http_request_text(&self.http_client, "GET", &request_url, &request_headers, &None).await?;
    file::write_text_to_file(&format!("./data/polygon-{symbol}-{resolution}-{from_timestamp}-{to_timestamp}.json"), &response_body).await;
    let parsed_response_body: PolygonResponseRoot = serde_json::from_str(&response_body)?;
    let mut candles = vec![];
    for result in parsed_response_body.results {
      let timestamp = result.t / 1000;
      let open = result.o;
      let high = result.h;
      let low = result.l;
      let close = result.c;
      let volume = result.v as i64;
      candles.push(Candle {
        //symbol: symbol.to_string(),
        //resolution: resolution.to_string(),
        timestamp,
        open,
        high,
        low,
        close,
        volume,
      });
    }
    // TODO: too cheap to pay $30/mo so 5 requests per minute?
    tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;
    return Ok(candles);
  }
}
