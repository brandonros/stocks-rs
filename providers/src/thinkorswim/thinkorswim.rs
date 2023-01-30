use super::structs::*;
use async_shutdown::Shutdown;
use common::structs::Candle;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use json_dotpath::DotPaths;
use log::{info, trace};
use serde_json::Value;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use uuid::Uuid;
use websocket_lite::{Message, Opcode};

pub struct ThinkOrSwim {
  pub rt_handle: Handle,
  pub buffer_arc: Arc<Mutex<Vec<Value>>>,
  pub sender: UnboundedSender<Value>,
  pub receiver: Arc<Mutex<UnboundedReceiver<Value>>>,
  pub shutdown: Shutdown,
}

impl ThinkOrSwim {
  pub fn new() -> ThinkOrSwim {
    let rt_handle = tokio::runtime::Handle::current();
    let shutdown = async_shutdown::Shutdown::new();
    // buffer
    let buffer_arc = Arc::new(Mutex::new(vec![]));
    let local_buffer_arc = buffer_arc.clone();
    // channel
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Value>();
    return ThinkOrSwim {
      rt_handle,
      buffer_arc: local_buffer_arc,
      sender,
      receiver: Arc::new(Mutex::new(receiver)),
      shutdown,
    };
  }

  pub async fn connect(&self) {
    // connect
    let url = "wss://services.thinkorswim.com/Services/WsJson";
    let ws_stream = websocket_lite::ClientBuilder::new(url).unwrap().async_connect().await.unwrap();
    let (mut ws_sink, mut ws_stream) = ws_stream.split();
    // mpsc recv -> websocket send thread
    let local_receiver = self.receiver.clone();
    let local_shutdown = self.shutdown.clone();
    self.rt_handle.spawn(async move {
      loop {
        let mut receiver = local_receiver.lock().await;
        let wrapped_recv = local_shutdown.wrap_cancel(receiver.recv()).await;
        if wrapped_recv.is_none() {
          info!("ws closing");
          ws_sink.send(Message::close(None)).await.unwrap();
          receiver.close();
          return;
        }
        let message = wrapped_recv.unwrap();
        if message.is_none() {
          return;
        }
        let message = message.unwrap();
        let stringified_message = message.to_string();
        trace!("ws sending: {:?}", stringified_message);
        ws_sink.send(Message::text(stringified_message)).await.unwrap();
      }
    });
    // websocket recv -> push to buffer thread
    let local_shutdown = self.shutdown.clone();
    let local_buffer_arc = self.buffer_arc.clone();
    self.rt_handle.spawn(async move {
      loop {
        let wrapped_send = local_shutdown.wrap_cancel(ws_stream.next()).await;
        if wrapped_send.is_none() {
          return;
        }
        let message = wrapped_send.unwrap().unwrap().unwrap();
        if message.opcode() == Opcode::Close {
          return;
        }
        let message = message.as_text().unwrap();
        let parsed_message: Value = serde_json::from_str(&message).unwrap();
        // skip heartbeat
        if parsed_message.as_object().unwrap().get("heartbeat").is_some() {
          continue;
        }
        trace!("ws received: {:#?}", parsed_message);
        local_buffer_arc.lock().await.push(parsed_message);
      }
    });
  }

  pub async fn pluck_response_by_callback(&self, mut cb: impl FnMut(&Value) -> bool) -> Option<Value> {
    let mut buffer_guard = self.buffer_arc.lock().await;
    let index = buffer_guard.iter().position(|value| {
      return cb(value);
    });
    if index.is_some() {
      let index = index.unwrap();
      let element = buffer_guard.get(index).unwrap().to_owned();
      buffer_guard.remove(index);
      return Some(element);
    }
    return None;
  }

  pub async fn wait_for_response_by_callback(&self, mut cb: impl FnMut(&Value) -> bool, timeout_ms: u128) -> Option<Value> {
    let start = tokio::time::Instant::now();
    loop {
      let elapsed = start.elapsed().as_millis();
      if elapsed >= timeout_ms {
        return None;
      }
      let response = self.pluck_response_by_callback(&mut cb).await;
      if response.is_some() {
        return response;
      }
      // sleep
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
  }

  pub async fn wait_for_responses_by_callback(&self, mut cb: impl FnMut(&Value) -> bool, timeout_ms: u128) -> Vec<Value> {
    let mut results = vec![];
    let start = tokio::time::Instant::now();
    loop {
      let elapsed = start.elapsed().as_millis();
      if elapsed >= timeout_ms {
        break;
      }
      let response = self.pluck_response_by_callback(&mut cb).await;
      if response.is_some() {
        results.push(response.unwrap());
      }
      // sleep
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    return results;
  }

  pub async fn wait_for_response_by_message_id(&self, message_id: String, timeout_ms: u128) -> Option<Value> {
    let mut cb = |element: &Value| {
      let element_message_id = element.dot_get::<String>("payload.0.header.id").unwrap();
      return element_message_id.is_some() && element_message_id.unwrap() == message_id;
    };
    return self.wait_for_response_by_callback(&mut cb, timeout_ms).await;
  }

  pub async fn wait_for_responses_by_message_id(&self, message_id: String, timeout_ms: u128) -> Vec<Value> {
    let mut cb = |element: &Value| {
      let element_message_id = element.dot_get::<String>("payload.0.header.id").unwrap();
      return element_message_id.is_some() && element_message_id.unwrap() == message_id;
    };
    return self.wait_for_responses_by_callback(&mut cb, timeout_ms).await;
  }

  pub async fn init_heartbeat(&self) {
    let message = serde_json::json!({
      "ver": "27.*.*",
      "fmt": "json-patches-structured",
      "heartbeat": "2s"
    });
    self.sender.send(message).unwrap();
  }

  pub async fn login(&self, token: String) -> Result<Value, String> {
    info!("login");
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "login",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "domain": "TOS",
            "platform": "PROD",
            "token": token,
            "accessToken": "",
            "tag": "TOSWeb"
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await;
    if response.is_none() {
      return Err(format!("timed out"));
    }
    let body = response.unwrap().dot_get::<Value>("payload.0.body").unwrap().unwrap();
    let authentication_status = body.dot_get::<String>("authenticationStatus");
    if authentication_status.is_err() {
      return Err(format!("invalid response"));
    }
    let authentication_status = authentication_status.unwrap().unwrap();
    if authentication_status != "OK" {
      return Err(format!("invalid response"));
    }
    return Ok(body);
  }

  pub async fn get_option_series(&self, underlying: String) -> Result<Vec<OptionSeries>, String> {
    info!("get_option_series underlying = {}", underlying);
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "id": message_id,
            "service": "optionSeries",
            "ver": 0
          },
          "params": {
            "underlying": underlying
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await;
    if response.is_none() {
      return Err(format!("timed out"));
    }
    let response = response.unwrap();
    return Ok(response.dot_get::<Vec<OptionSeries>>("payload.0.body.series").unwrap().unwrap());
  }

  pub async fn get_option_chain(&self, underlying: String, series_name: String) -> Result<OptionChain, String> {
    info!("get_option_chain underlying = {} series_name = {}", underlying, series_name);
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "option_chain/get",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "underlyingSymbol": underlying,
            "filter": {
              "strikeQuantity": 2147483647,
              "seriesNames": [
                series_name
              ]
            }
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await;
    if response.is_none() {
      return Err(format!("timed out"));
    }
    let response = response.unwrap();
    return Ok(response.dot_get::<OptionChain>("payload.0.body.optionSeries.0").unwrap().unwrap());
  }

  // TODO: use implied volatility/series expected move at the expiration date level somewhere?
  pub async fn _get_option_series_quotes(&self, underlying: String) -> Vec<OptionSeriesQuote> {
    info!("get_option_series_quotes underlying = {}", underlying);
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "optionSeries/quotes",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "underlying": underlying,
            "exchange": "BEST",
            "fields": [
              "IMPLIED_VOLATILITY",
              "SERIES_EXPECTED_MOVE"
            ]
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await.unwrap();
    return response.dot_get::<Vec<OptionSeriesQuote>>("payload.0.body.series").unwrap().unwrap();
  }

  pub async fn get_option_chain_quotes(
    &self,
    underlying: String,
    series_name: String,
    min_strike: usize,
    max_strike: usize,
  ) -> Result<Vec<OptionChainQuote>, String> {
    info!(
      "get_option_chain_quotes underlying = {} series_name = {} min_strike = {} max_strike = {}",
      underlying, series_name, min_strike, max_strike
    );
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "quotes/options",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "underlyingSymbol": underlying,
            "exchange": "BEST",
            "fields": [
              "BID",
              "ASK",
              "PROBABILITY_ITM",
              "DELTA",
              "OPEN_INT",
              "VOLUME",
              "IMPLIED_VOLATILITY",
              "GAMMA",
              "INTRINSIC",
              "EXTRINSIC",
              "LAST",
              "MARK",
              "MARK_CHANGE",
              "MARK_PERCENT_CHANGE",
              "PROBABILITY_OTM",
              "RHO",
              "THEO_PRICE",
              "THETA",
              "VEGA"
            ],
            "filter": {
              "seriesNames": [
                series_name
              ],
              "minStrike": min_strike,
              "maxStrike": max_strike
            }
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    // wait for responses
    let responses = self.wait_for_responses_by_message_id(message_id.to_owned(), 5000).await;
    // get initial snapshot
    let initial_response = responses.iter().find(|response| {
      let header_type = response.dot_get::<String>("payload.0.header.type").unwrap().unwrap();
      return header_type == "snapshot";
    });
    if initial_response.is_none() {
      return Err(format!("failed to get option chain quote initial response"));
    }
    // get patch response
    let patch_response = responses.iter().find(|response| {
      let has_patches_op = response.dot_has_checked("payload.0.body.patches.0.op").unwrap();
      if has_patches_op == false {
        return false;
      }
      let patch_op = response.dot_get::<String>("payload.0.body.patches.0.op").unwrap().unwrap();
      let patch_path = response.dot_get::<String>("payload.0.body.patches.0.path").unwrap().unwrap();
      return patch_op == "replace" && patch_path == "";
    });
    if patch_response.is_none() {
      return Err(format!(
        "failed to get option chain quote patch response for {}:{}:{}:{}",
        underlying, series_name, min_strike, max_strike
      ));
    }
    let patch_response = patch_response.unwrap();
    return Ok(
      patch_response
        .dot_get::<Vec<OptionChainQuote>>("payload.0.body.patches.0.value.items")
        .unwrap()
        .unwrap(),
    );
  }

  pub async fn get_quote(&self, symbol: String) -> Result<Quote, String> {
    info!("get_quote symbol = {}", symbol);
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "quotes",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "account": "COMBINED ACCOUNT",
            "symbols": [
              symbol
            ],
            "refreshRate": 300,
            "fields": [
              "ASK",
              "ASK_EXCHANGE",
              "ASK_SIZE",
              "BACK_VOLATILITY",
              "BETA",
              "BID",
              "BID_EXCHANGE",
              "BID_SIZE",
              "BORROW_STATUS",
              "CLOSE",
              "DELTA",
              "DIV_AMOUNT",
              "EPS",
              "EXD_DIV_DATE",
              "FRONT_VOLATILITY",
              "GAMMA",
              "HIGH",
              "HIGH52",
              "HISTORICAL_VOLATILITY_30_DAYS",
              "IMPLIED_VOLATILITY",
              "INITIAL_MARGIN",
              "LAST",
              "LAST_EXCHANGE",
              "LAST_SIZE",
              "LOW",
              "LOW52",
              "MARK",
              "MARK_CHANGE",
              "MARK_PERCENT_CHANGE",
              "MARKET_CAP",
              "MARKET_MAKER_MOVE",
              "NET_CHANGE",
              "NET_CHANGE_PERCENT",
              "OPEN",
              "PE",
              "PERCENTILE_IV",
              "PUT_CALL_RATIO",
              "RHO",
              "THETA",
              "VEGA",
              "VOLATILITY_DIFFERENCE",
              "VOLATILITY_INDEX",
              "VOLUME",
              "VWAP",
              "YIELD",
            ]
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await;
    if response.is_none() {
      return Err(format!("timed out"));
    }
    let response = response.unwrap();
    return Ok(response.dot_get::<Quote>("payload.0.body.items.0").unwrap().unwrap());
  }

  pub async fn get_candles(&self, symbol: String) -> Result<Vec<Candle>, String> {
    info!("get_quote symbol = {}", symbol);
    let message_id = format!("{}", Uuid::new_v4());
    let message = serde_json::json!({
      "payload": [
        {
          "header": {
            "service": "chart",
            "id": message_id,
            "ver": 0
          },
          "params": {
            "symbol": symbol,
            "timeAggregation": "MIN1", // TODO: support different resolutions
            "studies": [],
            "range": "TODAY", // TODO: support different ranges
            "extendedHours": false // TODO: support different hours?
          }
        }
      ]
    });
    self.sender.send(message).unwrap();
    let response = self.wait_for_response_by_message_id(message_id, 5000).await;
    if response.is_none() {
      return Err(format!("timed out"));
    }
    let response = response.unwrap();
    // TODO: map candles
    panic!("TODO");
  }
}
