use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use yahoo_finance_api as yahoo;

use crate::structs::Candle;

pub async fn get_candles(symbol: &str, resolution: &str, start: DateTime<Tz>, end: DateTime<Tz>) -> Result<Vec<Candle>, String> {
  let utc_start = start.with_timezone(&Utc);
  let utc_end = end.with_timezone(&Utc);
  let interval = format!("{}m", resolution);
  let provider = yahoo::YahooConnector::new();
  let response = provider.get_quote_history_interval(symbol, utc_start, utc_end, &interval).await;
  if response.is_err() {
    return Err(format!("{:?}", response));
  }
  let response = response.unwrap();
  let quotes = response.quotes();
  if quotes.is_err() {
    return Err(format!("{:?}", quotes));
  }
  let quotes = quotes.unwrap();
  let candles: Vec<Candle> = quotes
    .iter()
    .map(|quote| {
      return Candle {
        symbol: symbol.to_string(),
        resolution: resolution.to_string(),
        timestamp: quote.timestamp as i64,
        open: quote.open,
        high: quote.high,
        low: quote.low,
        close: quote.close,
        volume: quote.volume as i64,
      };
    })
    .collect();
  return Ok(candles);
}
