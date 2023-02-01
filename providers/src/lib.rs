pub mod finnhub;
pub mod polygon;
pub mod robinhood;
pub mod thinkorswim;
pub mod tradingview;
pub mod yahoo_finance;

use std::str::FromStr;

use chrono::DateTime;
use chrono_tz::Tz;
use common::structs::Candle;

#[derive(Debug)]
pub enum Provider {
  Finnhub,
  YahooFinance,
  Polygon,
  Robinhood,
  ThinkOrSwim,
  TradingView,
}

impl FromStr for Provider {
  type Err = ();

  fn from_str(s: &str) -> Result<Provider, ()> {
    match s {
      "finnhub" => Ok(Provider::Finnhub),
      "yahoo_finance" => Ok(Provider::YahooFinance),
      "polygon" => Ok(Provider::Polygon),
      "robinhood" => Ok(Provider::Robinhood),
      "thinkorswim" => Ok(Provider::ThinkOrSwim),
      "tradingview" => Ok(Provider::TradingView),
      _ => Err(()),
    }
  }
}

// TODO: convert this to a trait?
pub async fn get_candles_by_provider_name(
  provider_name: &str,
  symbol: &str,
  resolution: &str,
  from: DateTime<Tz>,
  to: DateTime<Tz>,
) -> Result<Vec<Candle>, String> {
  match provider_name {
    "yahoo_finance" => {
      let provider = self::yahoo_finance::YahooFinance::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    "finnhub" => {
      let provider = self::finnhub::Finnhub::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    "polygon" => {
      let provider = self::polygon::Polygon::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    "tradingview" => {
      // TODO: make format consistent?
      let auth_token = String::from("unauthorized_user_token");
      let provider = self::tradingview::TradingView::new();
      let result = provider
        .get_candles(auth_token, String::from(symbol), String::from(resolution), 1, String::from("regular"), 500)
        .await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    _ => unimplemented!(),
  }
}
