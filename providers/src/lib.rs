pub mod finnhub;
pub mod polygon;
pub mod robinhood;
pub mod thinkorswim;
pub mod tradingview;
pub mod yahoo_finance;

use std::str::FromStr;

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
