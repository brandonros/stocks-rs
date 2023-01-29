use crate::structs::*;
use chrono::TimeZone;
use chrono::{NaiveDateTime, Utc};
use chrono_tz::US::Eastern;
use common::{greeks, json_time, structs::*};
use itertools::Itertools;
use log::warn;
use serde::Serialize;

pub fn build_minimal_snapshots(
  symbol: String,
  quote: Quote,
  option_chain: OptionChain,
  options_quotes: Vec<OptionChainQuote>,
  series_name: String,
  expiration_date: String,
  risk_free_rate: f64,
) -> Vec<MinimalSnapshot> {
  let now = Utc::now();
  let naive_now = now.naive_utc();
  let eastern_now = Eastern.from_utc_datetime(&naive_now);
  let mut snapshots = vec![];
  // format
  let strike_prices: Vec<f64> = option_chain
    .optionPairs
    .iter()
    .unique_by(|option_pair| {
      return format!("{}", option_pair.strike); // f64 unique weirdness workaround
    })
    .map(|option_pair| {
      return option_pair.strike;
    })
    .collect::<Vec<_>>();
  for strike_price in strike_prices {
    let option_pair = option_chain
      .optionPairs
      .iter()
      .find(|option_pair| {
        return option_pair.strike == strike_price && option_chain.name == series_name;
      })
      .unwrap();
    let call_row_quote = options_quotes.iter().find(|option_quote| {
      return option_quote.symbol == option_pair.callSymbol;
    });
    let put_row_quote = options_quotes.iter().find(|option_quote| {
      return option_quote.symbol == option_pair.putSymbol;
    });
    // skip rows missing quotes
    if call_row_quote.is_none() || put_row_quote.is_none() {
      warn!("skipping {} {} {}: missing quote", series_name, expiration_date, strike_price);
      continue;
    }
    let call_row_quote = call_row_quote.unwrap();
    let put_row_quote = put_row_quote.unwrap();
    // skip rows missing gamma?
    if call_row_quote.values.GAMMA.is_none() || put_row_quote.values.GAMMA.is_none() {
      warn!("skipping {} {} {}: missing gamma", series_name, expiration_date, strike_price);
      continue;
    }
    // parse strings -> f64
    let parsed_strike_price = strike_price;
    let parsed_underlying_last_trade_price = quote.values.LAST.unwrap();
    let parsed_underlying_ask_price = quote.values.ASK.unwrap();
    let parsed_underlying_bid_price = quote.values.BID.unwrap();
    let parsed_underlying_mark_price = (parsed_underlying_ask_price + parsed_underlying_bid_price) / 2.0;
    let parsed_call_last_trade_price = call_row_quote.values.LAST.unwrap();
    let parsed_put_last_trade_price = put_row_quote.values.LAST.unwrap();
    let parsed_put_volume = if put_row_quote.values.VOLUME.is_none() {
      0
    } else {
      put_row_quote.values.VOLUME.unwrap() as u32
    };
    let parsed_call_bid_price = call_row_quote.values.BID.unwrap();
    let parsed_call_ask_price = call_row_quote.values.ASK.unwrap();
    let parsed_call_mark_price = (parsed_call_bid_price + parsed_call_ask_price) / 2.0;
    let parsed_call_volume = if call_row_quote.values.VOLUME.is_none() {
      0
    } else {
      call_row_quote.values.VOLUME.unwrap() as u32
    };
    let parsed_put_bid_price = put_row_quote.values.BID.unwrap();
    let parsed_put_ask_price = put_row_quote.values.ASK.unwrap();
    let parsed_put_mark_price = (parsed_put_bid_price + parsed_put_ask_price) / 2.0;
    let parsed_call_gamma = call_row_quote.values.GAMMA.unwrap();
    let parsed_put_gamma = put_row_quote.values.GAMMA.unwrap();
    let parsed_call_implied_volatility = call_row_quote.values.IMPLIED_VOLATILITY.unwrap();
    let parsed_put_implied_volatility = put_row_quote.values.IMPLIED_VOLATILITY.unwrap();
    // parse expiration_date
    let parsed_expiration_date = chrono::DateTime::parse_from_rfc3339(&expiration_date).unwrap();
    let naive_parsed_expiration_date = parsed_expiration_date.naive_utc();
    let eastern_parsed_expiration_date = parsed_expiration_date.with_timezone(&Eastern);
    // second order greeks
    let call_second_order_greeks = greeks::calculate_greeks(
      eastern_now,
      eastern_parsed_expiration_date,
      parsed_underlying_mark_price,
      parsed_strike_price,
      parsed_call_implied_volatility,
      risk_free_rate,
    );
    let put_second_order_greeks = greeks::calculate_greeks(
      eastern_now,
      eastern_parsed_expiration_date,
      parsed_underlying_mark_price,
      parsed_strike_price,
      parsed_put_implied_volatility,
      risk_free_rate,
    );
    // build
    snapshots.push(MinimalSnapshot {
      // categorize
      source: String::from("thinkorswim"),
      symbol: symbol.to_owned(),
      expiration_date: naive_parsed_expiration_date,
      scraped_at: naive_now,
      strike_price: parsed_strike_price,
      // underlying
      underlying_last_trade_price: parsed_underlying_last_trade_price,
      underlying_mark_price: parsed_underlying_mark_price,
      // call
      call_delta: call_row_quote.values.DELTA.unwrap(),
      call_gamma: parsed_call_gamma,
      call_implied_volatility: parsed_call_implied_volatility,
      call_last_trade_price: parsed_call_last_trade_price,
      call_mark_price: parsed_call_mark_price,
      call_open_interest: call_row_quote.values.OPEN_INT.unwrap() as u32,
      call_rho: call_row_quote.values.RHO.unwrap(),
      call_theta: call_row_quote.values.THETA.unwrap(),
      call_vega: call_row_quote.values.VEGA.unwrap(),
      call_vanna: call_second_order_greeks.call_vanna,
      call_vomma: call_second_order_greeks.call_vomma,
      call_charm: call_second_order_greeks.call_charm,
      call_volume: parsed_call_volume,
      // put
      put_delta: put_row_quote.values.DELTA.unwrap(),
      put_gamma: parsed_put_gamma,
      put_implied_volatility: parsed_put_implied_volatility,
      put_last_trade_price: parsed_put_last_trade_price,
      put_mark_price: parsed_put_mark_price,
      put_open_interest: put_row_quote.values.OPEN_INT.unwrap() as u32,
      put_rho: put_row_quote.values.RHO.unwrap(),
      put_theta: put_row_quote.values.THETA.unwrap(),
      put_vega: put_row_quote.values.VEGA.unwrap(),
      put_vanna: put_second_order_greeks.put_vanna,
      put_vomma: put_second_order_greeks.put_vomma,
      put_charm: put_second_order_greeks.put_charm,
      put_volume: parsed_put_volume,
    });
  }
  return snapshots;
}
