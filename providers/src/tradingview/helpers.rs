use super::formatter;
use super::structs::*;
use super::tradingview::*;
use itertools::Itertools;
use json_dotpath::DotPaths;
use log::info;
use log::trace;
use serde_json::Value;

fn extract_prices_from_formatted_messages(formatted_messages: &[TradingViewMessage]) -> (f64, f64) {
  // calculate mark from bid/ask
  let qsd_bid_ask_messages: Vec<TradingViewMessage> = formatted_messages
    .iter()
    .cloned()
    .filter(|message| {
      return message.message_type == TradingViewMessageType::QsdBidAsk;
    })
    .sorted_by(|a, b| {
      return b.timestamp.partial_cmp(&a.timestamp).unwrap();
    })
    .collect();
  assert!(qsd_bid_ask_messages.len() > 0);
  let most_recent_bid_ask_message = qsd_bid_ask_messages.last().unwrap();
  let bid = most_recent_bid_ask_message.value.dot_get::<f64>("p.1.v.bid").unwrap().unwrap();
  let ask = most_recent_bid_ask_message.value.dot_get::<f64>("p.1.v.ask").unwrap().unwrap();
  let mark_price = (bid + ask) / 2.0;
  info!("mark_price = {}", mark_price);
  // calculate last price from QsdSessionHolidays
  let qsd_session_holidays_messages: Vec<TradingViewMessage> = formatted_messages
    .iter()
    .cloned()
    .filter(|message| {
      return message.message_type == TradingViewMessageType::QsdSessionHolidays;
    })
    .sorted_by(|a, b| {
      return b.timestamp.partial_cmp(&a.timestamp).unwrap();
    })
    .collect();
  assert!(qsd_session_holidays_messages.len() > 0);
  let most_recent_session_holidays_message = qsd_session_holidays_messages.last().unwrap();
  let last_price = most_recent_session_holidays_message.value.dot_get::<f64>("p.1.v.lp").unwrap().unwrap();
  //let extended_session_last_price = most_recent_session_holidays_message.value.dot_get::<f64>("p.1.v.rp").unwrap().unwrap(); // TODO: what does this stand for?
  return (mark_price, last_price);
}

pub async fn indicator_job_cb(
  symbol: String,
  indicator_name: String,
  timeframe: String,
  range: usize,
  session_type: String,
  indicator_study_values: &Value,
) -> IndicatorSnapshot {
  let exchange = if symbol == "SPY" { String::from("AMEX") } else { panic!("TODO") };
  let auth_token = std::env::var("TRADINGVIEW_AUTH_TOKEN").unwrap();
  let now = chrono::Utc::now().timestamp();
  let quote_session_id = format!("qs_QUOTE_SESSIONID_{}", now);
  let chart_session_id = format!("cs_{}", now);
  let formatted_symbol = format!("{}:{}", exchange, symbol);
  let study_symbol_settings = format!("={}", serde_json::json!({ "session": session_type, "symbol": formatted_symbol }).to_string()); // TODO: adjustment splits / currency ID?
  let quote_symbol_settings = format!("={}", serde_json::json!({ "session": session_type, "symbol": formatted_symbol }).to_string());
  let series_parent_id = "series_parent_id";
  let chart_symbol_id = "symbol_id";
  let series_id = "series_id";
  let study_id = "study_id";
  let study_parent_id = "study_parent_id";
  let study_script_id = "Script@tv-scripting-101!";
  // check JWT expiration
  //jwt::check_jwt_expiration(auth_token.to_owned()); // TODO: weird thing where their server accepts expired JWT?
  // kickoff
  let tradingview = TradingView::new();
  tradingview.connect().await.unwrap();
  tradingview.set_auth_token(&auth_token);
  // quote
  tradingview.create_quote_session(&quote_session_id);
  tradingview.add_quote_session_symbol(&quote_session_id, &quote_symbol_settings);
  // indicator
  tradingview.chart_create_session(&chart_session_id);
  tradingview.resolve_symbol(&chart_session_id, chart_symbol_id, &study_symbol_settings);
  tradingview.create_series(&chart_session_id, series_parent_id, series_id, chart_symbol_id, &timeframe, range);
  tradingview.create_study(
    &chart_session_id,
    study_parent_id,
    study_id,
    series_parent_id,
    study_script_id,
    indicator_study_values,
  );
  // sleep to allow websocket message buffer to fill
  let sleep_ms = 5000;
  tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
  // shutdown
  tradingview.shutdown.shutdown();
  // format message
  let formatted_messages = tradingview.format_buffer_messages().await;
  trace!("{}", serde_json::to_string(&formatted_messages).unwrap());
  // extract prices
  let (mark_price, last_price) = extract_prices_from_formatted_messages(&formatted_messages);
  // calculate direction
  let du_study_messages: Vec<TradingViewMessage> = formatted_messages
    .iter()
    .cloned()
    .filter(|message| {
      return message.message_type == TradingViewMessageType::DuStudy;
    })
    .collect();
  assert!(du_study_messages.len() > 0);
  let indicator_snapshots = formatter::process_du_study_messages(
    &du_study_messages,
    symbol.to_owned(),
    indicator_name.to_owned(),
    timeframe.to_owned(),
    session_type.to_owned(),
    mark_price,
    last_price,
  );
  assert!(indicator_snapshots.len() > 0);
  let most_recent_indicator_snapshot = indicator_snapshots.last().unwrap();
  info!("most_recent_indicator_snapshot = {:?}", most_recent_indicator_snapshot);
  // return
  return most_recent_indicator_snapshot.to_owned();
}
