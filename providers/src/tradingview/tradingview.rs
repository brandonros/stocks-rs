use super::structs::*;
use async_shutdown::Shutdown;
use common::structs::Candle;
use futures::{SinkExt, StreamExt};
use json_dotpath::DotPaths;
use log::{debug, trace, warn};
use serde_json::Value;
use std::sync::Arc;
use tokio::{
  runtime::Handle,
  sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
  },
};
use websocket_lite::{Message, Opcode};

pub struct TradingView {
  pub rt_handle: Handle,
  pub buffer_arc: Arc<Mutex<Vec<Value>>>,
  pub sender: UnboundedSender<Value>,
  pub receiver: Arc<Mutex<UnboundedReceiver<Value>>>,
  pub shutdown: Shutdown,
}

impl TradingView {
  pub fn new() -> TradingView {
    let rt_handle = tokio::runtime::Handle::current();
    let shutdown = async_shutdown::Shutdown::new();
    // buffer
    let buffer_arc = Arc::new(Mutex::new(vec![]));
    let local_buffer_arc = buffer_arc.clone();
    // channel
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Value>();
    return TradingView {
      rt_handle,
      buffer_arc: local_buffer_arc,
      sender,
      receiver: Arc::new(Mutex::new(receiver)),
      shutdown,
    };
  }

  pub async fn connect(&self) -> Result<(), String> {
    // websocket
    let url = String::from("wss://data.tradingview.com/socket.io/websocket");
    let mut ws_builder = websocket_lite::ClientBuilder::new(&url).unwrap();
    ws_builder.add_header(String::from("Origin"), String::from("https://s.tradingview.com"));
    let connect_result = ws_builder.async_connect().await;
    if connect_result.is_err() {
      return Err(format!("{:?}", connect_result.err()));
    }
    let ws = connect_result.unwrap();
    let (mut ws_sink, mut ws_stream) = ws.split::<Message>();
    // mpsc recv -> websocket send thread
    let local_receiver = self.receiver.clone();
    let local_shutdown = self.shutdown.clone();
    let _mpsc_recv_handle = self.rt_handle.spawn(async move {
      loop {
        let mut receiver = local_receiver.lock().await;
        let wrapped_mpsc_recv = local_shutdown.wrap_cancel(receiver.recv()).await;
        if wrapped_mpsc_recv.is_none() {
          warn!("mpsc_receiver: ws closing?");
          ws_sink.send(Message::close(None)).await.unwrap();
          receiver.close();
          return;
        }
        let mpsc_message = wrapped_mpsc_recv.unwrap();
        if mpsc_message.is_none() {
          warn!("mpsc_receiver received none?");
          return;
        }
        let mpsc_message = mpsc_message.unwrap();
        let stringified_mpsc_payload = if mpsc_message.is_string() {
          mpsc_message.as_str().unwrap().to_string() // do not restringify simple ping packets which are not objects
        } else {
          serde_json::to_string(&mpsc_message).unwrap()
        };
        let formatted_ws_packet = format!("~m~{}~m~{}", stringified_mpsc_payload.len(), stringified_mpsc_payload);
        trace!("ws sending: {}", formatted_ws_packet);
        ws_sink.send(Message::text(formatted_ws_packet)).await.unwrap();
      }
    });
    // websocket recv -> push to buffer thread
    let local_shutdown = self.shutdown.clone();
    let local_buffer_arc = self.buffer_arc.clone();
    let _ws_recv_handle = self.rt_handle.spawn(async move {
      loop {
        let wrapped_ws_recv = local_shutdown.wrap_cancel(ws_stream.next()).await;
        if wrapped_ws_recv.is_none() {
          warn!("ws_recv is_none()");
          return;
        }
        let ws_message = wrapped_ws_recv.unwrap().unwrap().unwrap();
        if ws_message.opcode() == Opcode::Close {
          warn!("ws_recv closed");
          return;
        }
        assert_eq!(ws_message.opcode(), Opcode::Text);
        let ws_message_string = ws_message.as_text().unwrap();
        trace!("ws recv: {}", ws_message_string);
        let re = regex::Regex::new(r"~m~\d+~m~").unwrap();
        for ws_payload_string in re.split(&ws_message_string).into_iter() {
          // skip empty?
          if ws_payload_string.len() == 0 {
            continue;
          }
          let ping_re = regex::Regex::new(r"~h~(\d+)").unwrap();
          if ping_re.is_match(ws_payload_string) {
            let captures = ping_re.captures(ws_payload_string).unwrap();
            let id_string = captures.get(1).unwrap().as_str();
            let id = id_string.parse::<usize>().unwrap();
            local_buffer_arc.lock().await.push(serde_json::json!({
                "ping": {
                    "id": id
                }
            }));
          } else {
            trace!("ws parse: {}", ws_payload_string);
            let parsed_ws_payload: Value = serde_json::from_str(&ws_payload_string).unwrap();
            local_buffer_arc.lock().await.push(parsed_ws_payload);
          }
        }
      }
    });
    return Ok(());
  }

  pub fn set_auth_token(&self, auth_token: &str) {
    let packet = serde_json::json!({
        "m": "set_auth_token",
        "p": [
            auth_token
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn create_quote_session(&self, quote_session_id: &str) {
    let packet = serde_json::json!({
        "m": "quote_create_session",
        "p": [
            quote_session_id
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn add_quote_session_symbol(&self, quote_session_id: &str, symbol: &str) {
    let packet = serde_json::json!({
        "m": "quote_add_symbols",
        "p": [
            quote_session_id,
            symbol
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn resolve_symbol(&self, chart_session_id: &str, symbol_id: &str, settings: &str) {
    let packet = serde_json::json!({
        "m": "resolve_symbol",
        "p": [
            chart_session_id,
            symbol_id,
            settings
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn chart_create_session(&self, chart_session_id: &str) {
    let packet = serde_json::json!({
        "m": "chart_create_session",
        "p": [
            chart_session_id,
            ""
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn create_series(&self, chart_session_id: &str, series_parent_id: &str, series_id: &str, symbol_id: &str, timeframe: &str, range: usize) {
    let packet = serde_json::json!({
        "m": "create_series",
        "p": [
            chart_session_id,
            series_parent_id,
            series_id,
            symbol_id,
            timeframe,
            range,
            ""
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn create_study(&self, chart_session_id: &str, study_parent_id: &str, study_id: &str, series_parent_id: &str, unk1: &str, study_values: &Value) {
    let packet = serde_json::json!({
        "m": "create_study",
        "p": [
            chart_session_id,
            study_parent_id, // st6
            study_id, // st1
            series_parent_id, // sds_1
            unk1, // Script@tv-scripting-101!
            study_values
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn remove_study(&self, chart_session_id: &str, study_parent_id: &str) {
    let packet = serde_json::json!({
        "m": "remove_study",
        "p": [
            chart_session_id,
            study_parent_id, // st6
        ]
    });
    self.sender.send(packet).unwrap();
  }

  pub fn pong(&self, ping_id: usize) {
    let value = format!("~h~{}", ping_id);
    let packet = serde_json::json!(value);
    self.sender.send(packet).unwrap();
  }

  pub async fn format_buffer_messages(&self) -> Vec<TradingViewMessage> {
    let buffer_guard = self.buffer_arc.lock().await;
    let messages: Vec<TradingViewMessage> = buffer_guard
      .iter()
      .map(|value| {
        if value.dot_has("ping.id") {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::Ping,
            value: value.to_owned(),
          };
        }
        if value.dot_has_checked("session_id").unwrap() {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::ServerInfo,
            value: value.to_owned(),
          };
        }
        let m = value.dot_get::<String>("m").unwrap().unwrap();
        if m == "qsd" {
          let is_qsd_founded = value.dot_has_checked("p.1.v.founded").unwrap();
          let is_qsd_session_holidays = value.dot_has_checked("p.1.v.session_holidays").unwrap();
          let is_qsd_bid_ask = value.dot_has_checked("p.1.v.bid_size").unwrap();
          let is_qsd_last_price = value.dot_has_checked("p.1.v.lp").unwrap();
          let is_qsd_volume = value.dot_has_checked("p.1.v.volume").unwrap();
          let is_qsd_rt_update_time = value.dot_has_checked("p.1.v.rt-update-time").unwrap();
          let is_qsd_fundamental_data = value.dot_has_checked("p.1.v.fundamental_data").unwrap();
          let is_qsd_empty_trade_loaded = value.dot_has_checked("p.1.v.trade_loaded").unwrap();
          let is_qsd_after_market_last_price = value.dot_has_checked("p.1.v.rch").unwrap();
          let is_qsd_empty_rtc_time = value.dot_has_checked("p.1.v.rtc_time").unwrap();
          if is_qsd_founded {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdFounded,
              value: value.to_owned(),
            };
          } else if is_qsd_session_holidays {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdSessionHolidays,
              value: value.to_owned(),
            };
          } else if is_qsd_bid_ask {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdBidAsk,
              value: value.to_owned(),
            };
          } else if is_qsd_last_price {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdLastPrice,
              value: value.to_owned(),
            };
          } else if is_qsd_volume {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdVolume,
              value: value.to_owned(),
            };
          } else if is_qsd_rt_update_time {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdRtUpdateTime,
              value: value.to_owned(),
            };
          } else if is_qsd_fundamental_data {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdFundamentalData,
              value: value.to_owned(),
            };
          } else if is_qsd_empty_trade_loaded {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdEmptyTradeLoaded,
              value: value.to_owned(),
            };
          } else if is_qsd_empty_rtc_time {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdEmptyRtcTime,
              value: value.to_owned(),
            };
          } else if is_qsd_after_market_last_price {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::QsdAfterMarketLastPrice,
              value: value.to_owned(),
            };
          } else {
            panic!("unknown qsd message: {}", value.to_string());
          }
        } else if m == "du" {
          let is_du_series_parent = value.dot_has_checked("p.1.series_parent_id").unwrap();
          let is_du_study_parent = value.dot_has_checked("p.1.study_parent_id").unwrap();
          if is_du_series_parent {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::DuSeries,
              value: value.to_owned(),
            };
          } else if is_du_study_parent {
            return TradingViewMessage {
              timestamp: chrono::Utc::now().timestamp(),
              message_type: TradingViewMessageType::DuStudy,
              value: value.to_owned(),
            };
          } else {
            panic!("unknown du message: {}", value.to_string());
          }
        } else if m == "timescale_update" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::TimescaleUpdate,
            value: value.to_owned(),
          };
        } else if m == "study_completed" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::StudyCompleted,
            value: value.to_owned(),
          };
        } else if m == "series_completed" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::SeriesCompleted,
            value: value.to_owned(),
          };
        } else if m == "symbol_resolved" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::SymbolResolved,
            value: value.to_owned(),
          };
        } else if m == "quote_completed" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::QuoteCompleted,
            value: value.to_owned(),
          };
        } else if m == "series_loading" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::SeriesLoading,
            value: value.to_owned(),
          };
        } else if m == "study_loading" {
          return TradingViewMessage {
            timestamp: chrono::Utc::now().timestamp(),
            message_type: TradingViewMessageType::StudyLoading,
            value: value.to_owned(),
          };
        } else {
          panic!("unknown message type = {}", value.to_string());
        }
      })
      .collect();
    return messages;
  }

  pub async fn _ping_handler(&self) {
    loop {
      let messages = self.format_buffer_messages().await;
      // handle pings
      for message in messages.iter() {
        if message.message_type == TradingViewMessageType::Ping {
          let ping_id = message.value.dot_get::<usize>("ping.id").unwrap().unwrap();
          debug!("ping received: {}", ping_id);
          self.pong(ping_id);
        }
      }
      // TODO: delete them from buffer so they don't keep coming up?
      // sleep
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
  }

  pub async fn get_candles(&self, auth_token: String, symbol: String, timeframe: String, range: usize, session_type: String, buffer_fill_delay_ms: u64) -> Result<Vec<Candle>, String> {
    let exchange = if symbol == "SPY" { String::from("AMEX") } else { panic!("TODO") };
    let now = chrono::Utc::now().timestamp();
    let quote_session_id = format!("qs_QUOTE_SESSIONID_{}", now);
    let chart_session_id = format!("cs_{}", now);
    let formatted_symbol = format!("{}:{}", exchange, symbol);
    let study_symbol_settings = format!("={}", serde_json::json!({ "session": session_type, "symbol": formatted_symbol }).to_string()); // TODO: adjustment splits / currency ID?
    let quote_symbol_settings = format!("={}", serde_json::json!({ "session": session_type, "symbol": formatted_symbol }).to_string());
    let series_parent_id = "series_parent_id";
    let chart_symbol_id = "symbol_id";
    let series_id = "series_id";
    // check JWT expiration
    //jwt::check_jwt_expiration(auth_token.to_owned()); // TODO: weird thing where their server accepts expired JWT?
    // kickoff
    let result = self.connect().await;
    if result.is_err() {
      return Err(format!("{:?}", result));
    }
    self.set_auth_token(&auth_token);
    // quote
    self.create_quote_session(&quote_session_id);
    self.add_quote_session_symbol(&quote_session_id, &quote_symbol_settings);
    // indicator
    self.chart_create_session(&chart_session_id);
    self.resolve_symbol(&chart_session_id, chart_symbol_id, &study_symbol_settings);
    self.create_series(&chart_session_id, series_parent_id, series_id, chart_symbol_id, &timeframe, range);
    // sleep to allow websocket message buffer to fill
    tokio::time::sleep(tokio::time::Duration::from_millis(buffer_fill_delay_ms)).await;
    // shutdown
    self.shutdown.shutdown();
    // format message
    let formatted_messages = self.format_buffer_messages().await;
    trace!("{}", serde_json::to_string(&formatted_messages).unwrap());
    // extract + format candles
    let timescale_update_message = formatted_messages
      .iter()
      .find(|message| {
        return message.message_type == TradingViewMessageType::TimescaleUpdate;
      })
      .unwrap();
    let s = timescale_update_message.value.dot_get::<Vec<Value>>("p.1.series_parent_id.s").unwrap().unwrap();
    let formatted_candles: Vec<Candle> = s
      .iter()
      .map(|value| {
        let candle_timestamp = value.dot_get::<f64>("v.0").unwrap().unwrap();
        let open = value.dot_get::<f64>("v.1").unwrap().unwrap();
        let high = value.dot_get::<f64>("v.2").unwrap().unwrap();
        let low = value.dot_get::<f64>("v.3").unwrap().unwrap();
        let close = value.dot_get::<f64>("v.4").unwrap().unwrap();
        let volume = value.dot_get::<f64>("v.5").unwrap().unwrap();
        // handle weird .0 float problem from tradingview by casting to integers
        let candle_timestamp = candle_timestamp as i64;
        let volume = volume as usize;
        return Candle {
          //source: String::from("tradingview"), // TODO: sources on candles?
          symbol: symbol.to_owned(),
          resolution: timeframe.to_owned(),
          timestamp: candle_timestamp,
          open,
          high,
          low,
          close,
          volume: volume as i64
        };
      })
      .collect();
    trace!("{}", serde_json::to_string(&formatted_candles).unwrap());
    // return
    return Ok(formatted_candles);
  }
}
