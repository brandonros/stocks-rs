use crate::structs::*;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::US::Eastern;
use common::greeks;
use common::structs::*;
use itertools::Itertools;

pub fn build_minimal_snapshots(
  symbol: String,
  quote: Quote,
  options: Vec<OptionInstrument>,
  options_quotes: Vec<OptionMarketData>,
  expiration_date: String,
  risk_free_rate: f64,
) -> Vec<MinimalSnapshot> {
  let now = Utc::now();
  let naive_now = now.naive_utc();
  let eastern_now = Eastern.from_utc_datetime(&naive_now);
  let mut snapshots: Vec<MinimalSnapshot> = vec![];
  // format
  let strike_prices: Vec<String> = options
    .iter()
    .map(|option_instrument| {
      return option_instrument.strike_price.to_owned();
    })
    .unique()
    .collect();
  for strike_price in strike_prices {
    let call_row = options
      .iter()
      .find(|option| {
        return option.r#type == "call" && option.strike_price == strike_price;
      })
      .unwrap();
    let put_row = options
      .iter()
      .find(|option| {
        return option.r#type == "put" && option.strike_price == strike_price;
      })
      .unwrap();
    let call_row_quote = options_quotes.iter().find(|option_quote| {
      return option_quote.instrument_id == call_row.id;
    });
    let put_row_quote = options_quotes.iter().find(|option_quote| {
      return option_quote.instrument_id == put_row.id;
    });
    // skip options with no quotes
    if call_row_quote.is_none() || put_row_quote.is_none() {
      continue;
    }
    let call_row_quote = call_row_quote.unwrap().to_owned();
    let put_row_quote = put_row_quote.unwrap().to_owned();
    // skip quotes with no greeks
    if call_row_quote.delta.is_none() || put_row_quote.delta.is_none() {
      continue;
    }
    // skip quotes with no last trade prices
    if call_row_quote.last_trade_price.is_none() || put_row_quote.last_trade_price.is_none() {
      continue;
    }
    // parse strings -> f64
    let parsed_strike_price = strike_price.parse::<f64>().unwrap();
    let parsed_underlying_last_trade_price = quote.last_trade_price.parse::<f64>().unwrap();
    let parsed_underlying_ask_price = quote.ask_price.parse::<f64>().unwrap();
    let parsed_underlying_bid_price = quote.bid_price.parse::<f64>().unwrap();
    let parsed_underlying_mark_price = (parsed_underlying_ask_price + parsed_underlying_bid_price) / 2.0;
    let parsed_call_last_trade_price = call_row_quote.last_trade_price.unwrap().parse::<f64>().unwrap();
    let parsed_put_last_trade_price = put_row_quote.last_trade_price.unwrap().parse::<f64>().unwrap();
    let parsed_call_bid_price = call_row_quote.bid_price.parse::<f64>().unwrap();
    let parsed_call_ask_price = call_row_quote.ask_price.parse::<f64>().unwrap();
    let parsed_call_mark_price = (parsed_call_bid_price + parsed_call_ask_price) / 2.0;
    let parsed_put_bid_price = put_row_quote.bid_price.parse::<f64>().unwrap();
    let parsed_put_ask_price = put_row_quote.ask_price.parse::<f64>().unwrap();
    let parsed_put_mark_price = (parsed_put_bid_price + parsed_put_ask_price) / 2.0;
    let parsed_call_gamma = call_row_quote.gamma.unwrap().parse::<f64>().unwrap();
    let parsed_put_gamma = put_row_quote.gamma.unwrap().parse::<f64>().unwrap();
    let parsed_call_implied_volatility = call_row_quote.implied_volatility.unwrap().parse::<f64>().unwrap();
    let parsed_put_implied_volatility = put_row_quote.implied_volatility.unwrap().parse::<f64>().unwrap();
    // parse expiration_date
    let formatted_expiration_date = format!("{}T21:00:00Z", expiration_date);
    let parsed_expiration_date = chrono::DateTime::parse_from_rfc3339(&formatted_expiration_date).unwrap();
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
    // build result
    snapshots.push(MinimalSnapshot {
      // categorize
      source: String::from("robinhood"),
      symbol: symbol.to_owned(),
      expiration_date: naive_parsed_expiration_date,
      scraped_at: naive_now,
      strike_price: parsed_strike_price,
      // underlying
      underlying_last_trade_price: parsed_underlying_last_trade_price,
      underlying_mark_price: parsed_underlying_mark_price,
      // call
      call_delta: call_row_quote.delta.unwrap().parse::<f64>().unwrap(),
      call_gamma: parsed_call_gamma,
      call_implied_volatility: parsed_call_implied_volatility,
      call_last_trade_price: parsed_call_last_trade_price,
      call_mark_price: parsed_call_mark_price,
      call_open_interest: call_row_quote.open_interest,
      call_rho: call_row_quote.rho.unwrap().parse::<f64>().unwrap(),
      call_theta: call_row_quote.theta.unwrap().parse::<f64>().unwrap(),
      call_vega: call_row_quote.vega.unwrap().parse::<f64>().unwrap(),
      call_vanna: call_second_order_greeks.call_vanna,
      call_vomma: call_second_order_greeks.call_vomma,
      call_charm: call_second_order_greeks.call_charm,
      call_volume: call_row_quote.volume,
      // put
      put_delta: put_row_quote.delta.unwrap().parse::<f64>().unwrap(),
      put_gamma: parsed_put_gamma,
      put_implied_volatility: parsed_put_implied_volatility,
      put_last_trade_price: parsed_put_last_trade_price,
      put_mark_price: parsed_put_mark_price,
      put_open_interest: put_row_quote.open_interest,
      put_rho: put_row_quote.rho.unwrap().parse::<f64>().unwrap(),
      put_theta: put_row_quote.theta.unwrap().parse::<f64>().unwrap(),
      put_vega: put_row_quote.vega.unwrap().parse::<f64>().unwrap(),
      put_vanna: put_second_order_greeks.put_vanna,
      put_vomma: put_second_order_greeks.put_vomma,
      put_charm: put_second_order_greeks.put_charm,
      put_volume: put_row_quote.volume,
    });
  }
  return snapshots;
}
