pub mod finnhub;
pub mod polygon;
pub mod yahoo_finance;

use std::str::FromStr;

#[derive(Debug)]
pub enum Provider {
  Finnhub,
  YahooFinance,
  Polygon,
}

impl FromStr for Provider {
  type Err = ();

  fn from_str(s: &str) -> Result<Provider, ()> {
    match s {
      "finnhub" => Ok(Provider::Finnhub),
      "yahoo_finance" => Ok(Provider::YahooFinance),
      "polygon" => Ok(Provider::Polygon),
      _ => Err(()),
    }
  }
}
