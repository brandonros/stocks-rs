#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionSeries {
  pub expiration: String,
  pub expirationStyle: String,
  pub isEuropean: bool,
  pub lastTradeDate: String,
  pub multiplier: f64,
  pub name: String,
  pub settlementType: String,
  pub spc: f64,
  pub underlying: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionPair {
  pub callDisplaySymbol: String,
  pub callSymbol: String,
  pub putDisplaySymbol: String,
  pub putSymbol: String,
  pub strike: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionChain {
  pub contract: String,
  pub contractDisplay: String,
  pub daysToExpiration: usize,
  pub expiration: String,
  pub expirationString: String,
  pub fractionalType: String,
  pub name: String,
  pub optionPairs: Vec<OptionPair>,
  pub settlementType: String,
  pub spc: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionChainQuoteValues {
  pub ASK: Option<f64>,
  pub BID: Option<f64>,
  pub DELTA: Option<f64>,
  pub EXTRINSIC: Option<f64>,
  pub GAMMA: Option<f64>,
  pub IMPLIED_VOLATILITY: Option<f64>,
  pub INTRINSIC: Option<f64>,
  pub LAST: Option<f64>,
  pub MARK: Option<f64>,
  pub MARK_CHANGE: Option<f64>,
  pub MARK_PERCENT_CHANGE: Option<f64>,
  pub OPEN_INT: Option<f64>,
  pub PROBABILITY_ITM: Option<f64>,
  pub PROBABILITY_OTM: Option<f64>,
  pub RHO: Option<f64>,
  pub THEO_PRICE: Option<f64>,
  pub THETA: Option<f64>,
  pub VEGA: Option<f64>,
  pub VOLUME: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionChainQuote {
  pub symbol: String,
  pub values: OptionChainQuoteValues,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionSeriesQuoteValues {
  pub IMPLIED_VOLATILITY: Option<f64>,
  pub SERIES_EXPECTED_MOVE: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionSeriesQuote {
  pub name: String,
  pub values: OptionSeriesQuoteValues,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuoteValues {
  pub ASK: Option<f64>,
  pub BACK_VOLATILITY: Option<f64>,
  pub BETA: Option<f64>,
  pub BID: Option<f64>,
  pub BORROW_STATUS: Option<String>,
  pub CLOSE: Option<f64>,
  pub DELTA: Option<f64>,
  pub FRONT_VOLATILITY: Option<f64>,
  pub GAMMA: Option<f64>,
  pub HIGH: Option<f64>,
  pub HIGH52: Option<f64>,
  pub HISTORICAL_VOLATILITY_30_DAYS: Option<f64>,
  pub IMPLIED_VOLATILITY: Option<f64>,
  pub LAST: Option<f64>,
  pub LOW: Option<f64>,
  pub LOW52: Option<f64>,
  pub MARK: Option<f64>,
  pub MARKET_CAP: Option<f64>,
  pub MARKET_MAKER_MOVE: Option<f64>,
  pub MARK_CHANGE: Option<f64>,
  pub MARK_PERCENT_CHANGE: Option<f64>,
  pub NET_CHANGE: Option<f64>,
  pub NET_CHANGE_PERCENT: Option<f64>,
  pub OPEN: Option<f64>,
  pub PERCENTILE_IV: Option<f64>,
  pub PUT_CALL_RATIO: Option<f64>,
  pub RHO: Option<f64>,
  pub THETA: Option<f64>,
  pub VEGA: Option<f64>,
  pub VOLATILITY_DIFFERENCE: Option<f64>,
  pub VOLATILITY_INDEX: Option<f64>,
  pub VOLUME: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quote {
  pub isDelayed: bool,
  pub symbol: String,
  pub values: QuoteValues,
}
