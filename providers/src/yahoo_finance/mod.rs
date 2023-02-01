use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use common::structs::*;
use yahoo_finance_api as yahoo;

pub struct YahooFinance {
  connector: yahoo::YahooConnector,
}

impl YahooFinance {
  pub fn new() -> YahooFinance {
    return YahooFinance {
      connector: yahoo::YahooConnector::new(),
    };
  }

  pub async fn get_candles(&self, symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>, String> {
    let utc_start = from.with_timezone(&Utc);
    let utc_end = to.with_timezone(&Utc);
    let interval = format!("{}m", resolution);
    let response = self.connector.get_quote_history_interval(symbol, utc_start, utc_end, &interval).await;
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
          //symbol: symbol.to_string(),
          //resolution: resolution.to_string(),
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
}
