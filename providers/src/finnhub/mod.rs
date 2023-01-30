pub mod structs;

use chrono::DateTime;
use chrono_tz::Tz;
use common::http_client;
use common::structs::*;
use structs::*;

pub struct Finnhub {
  http_client: reqwest::Client,
}

impl Finnhub {
  pub fn new() -> Finnhub {
    return Finnhub {
      http_client: reqwest::Client::new(),
    };
  }

  pub async fn get_candles(&self, symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>, String> {
    log::info!("get_candles symbol = {} resolution = {} from = {} to = {}", symbol, resolution, from, to);
    let mut request_url = url::Url::parse("https://finnhub.io/api/v1/stock/candle").unwrap();
    request_url.query_pairs_mut().append_pair("symbol", symbol);
    request_url.query_pairs_mut().append_pair("resolution", resolution);
    request_url.query_pairs_mut().append_pair("from", &format!("{}", from.timestamp()));
    request_url.query_pairs_mut().append_pair("to", &format!("{}", to.timestamp()));
    let request_url = request_url.as_str().to_string();
    let finnhub_api_token = std::env::var("FINNHUB_API_TOKEN").unwrap();
    let request_headers = vec![(String::from("X-Finnhub-Token"), String::from(finnhub_api_token))];
    let result = http_client::http_request_json::<FinnhubStockCandlesResponse>(&self.http_client, "GET", &request_url, &request_headers, &None).await;
    if result.is_err() {
      return Err(result.err().unwrap());
    }
    let response_body = result.unwrap();
    let timestamps = &response_body.t;
    let opens = &response_body.o;
    let highs = &response_body.h;
    let lows = &response_body.l;
    let closes = &response_body.c;
    let volumes = &response_body.v;
    let num_timestamps = timestamps.len();
    let mut candles = vec![];
    for i in 0..num_timestamps {
      let timestamp = timestamps[i];
      let open = opens[i];
      let high = highs[i];
      let low = lows[i];
      let close = closes[i];
      let volume = volumes[i] as i64;
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
}
