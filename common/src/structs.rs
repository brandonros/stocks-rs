use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{database, json_time};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MinimalSnapshot {
  // categorize
  pub source: String,
  pub symbol: String,
  #[serde(with = "json_time")]
  pub expiration_date: NaiveDateTime,
  #[serde(with = "json_time")]
  pub scraped_at: NaiveDateTime,
  pub strike_price: f64,
  // call
  pub call_delta: f64,
  pub call_gamma: f64,
  pub call_implied_volatility: f64,
  pub call_last_trade_price: f64,
  pub call_mark_price: f64,
  pub call_open_interest: u32,
  pub call_rho: f64,
  pub call_theta: f64,
  pub call_vega: f64,
  pub call_vanna: f64,
  pub call_vomma: f64,
  pub call_charm: f64,
  pub call_volume: u32,
  // put
  pub put_delta: f64,
  pub put_gamma: f64,
  pub put_implied_volatility: f64,
  pub put_last_trade_price: f64,
  pub put_mark_price: f64,
  pub put_open_interest: u32,
  pub put_rho: f64,
  pub put_theta: f64,
  pub put_vega: f64,
  pub put_vanna: f64,
  pub put_vomma: f64,
  pub put_charm: f64,
  pub put_volume: u32,
  // underlying
  pub underlying_last_trade_price: f64,
  pub underlying_mark_price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candle {
  pub symbol: String,
  pub resolution: String,
  pub timestamp: i64,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

impl database::ToQuery for Candle {
  fn insert(&self) -> (&str, Vec<(&str, &dyn rusqlite::ToSql)>) {
    let query = "
        INSERT OR REPLACE INTO candles (
          symbol,
          resolution,
          scraped_at,
          timestamp,
          open,
          high,
          low,
          close,
          volume
      ) VALUES (
          :symbol,
          :resolution,
          strftime('%s', 'now'),
          :timestamp,
          :open,
          :high,
          :low,
          :close,
          :volume
      )
    ";
    let params = rusqlite::named_params! {
      ":symbol": self.symbol,
      ":resolution": self.resolution,
      ":timestamp": self.timestamp,
      ":open": self.open,
      ":high": self.high,
      ":low": self.low,
      ":close": self.close,
      ":volume": self.volume
    };
    return (query, params.to_vec());
  }
}
