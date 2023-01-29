use crate::structs::*;
use itertools::Itertools;
use json_dotpath::DotPaths;
use serde_json::Value;
use std::collections::HashMap;

pub fn process_du_study_messages(
  du_study_messages: &Vec<TradingViewMessage>,
  symbol: String,
  indicator_name: String,
  timeframe: String,
  session_type: String,
  mark_price: f64,
  last_price: f64,
) -> Vec<IndicatorSnapshot> {
  let mut results_map = HashMap::<i64, (i64, i64, &str)>::new();
  for du_study_message in du_study_messages.iter() {
    let message_values = du_study_message.value.dot_get::<Vec<Value>>("p.1.study_parent_id.st").unwrap();
    if message_values.is_none() {
      continue;
    }
    let message_values = message_values.unwrap();
    for message_value in message_values.iter() {
      let candle_index = message_value.dot_get::<i64>("i").unwrap().unwrap();
      // skip weird negative candles?
      if candle_index < 0 {
        continue;
      }
      // extract values based on indicator type
      let candle_values = message_value.dot_get::<Value>("v").unwrap().unwrap();
      let candle_timestamp = candle_values.dot_get::<f64>("0").unwrap().unwrap() as i64;
      if indicator_name == "VWAP/MVWAP/EMA CROSSOVER" {
        let long_entry = candle_values.dot_get::<f64>("3").unwrap().unwrap();
        let short_entry = candle_values.dot_get::<f64>("4").unwrap().unwrap();
        let direction = if long_entry == 1.0 {
          "long"
        } else if short_entry == 1.0 {
          "short"
        } else {
          "flat"
        };
        if direction == "flat" {
          continue;
        }
        results_map.insert(candle_index, (du_study_message.timestamp, candle_timestamp, direction));
      } else if indicator_name == "Supertrend crossover" {
        let long_entry = candle_values.dot_get::<f64>("1").unwrap().unwrap();
        let short_entry = candle_values.dot_get::<f64>("2").unwrap().unwrap();
        let direction = if long_entry == 1.0 {
          "long"
        } else if short_entry == 1.0 {
          "short"
        } else {
          "flat"
        };
        if direction == "flat" {
          continue;
        }
        results_map.insert(candle_index, (du_study_message.timestamp, candle_timestamp, direction));
      } else {
        panic!("TODO");
      }
    }
  }
  let results: Vec<IndicatorSnapshot> = results_map
    .iter()
    .sorted()
    .map(|(_key, value)| {
      let (message_timestamp, candle_timestamp, direction) = *value;
      return IndicatorSnapshot {
        source: String::from("tradingview"),
        symbol: symbol.to_owned(),
        timeframe: timeframe.to_owned(),
        indicator_name: indicator_name.to_owned(),
        session_type: session_type.to_owned(),
        scraped_at: chrono::NaiveDateTime::from_timestamp_opt(message_timestamp, 0).unwrap(),
        candle_timestamp: chrono::NaiveDateTime::from_timestamp_opt(candle_timestamp, 0).unwrap(),
        direction: String::from(direction),
        underlying_mark_price: mark_price,
        underlying_last_price: last_price,
      };
    })
    .collect();
  return results.to_owned();
}
