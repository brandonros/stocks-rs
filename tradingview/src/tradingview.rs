use crate::structs::*;
use async_shutdown::Shutdown;
use futures::{SinkExt, StreamExt};
use json_dotpath::DotPaths;
use log::{debug, trace, warn};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use websocket_lite::{Message, Opcode};

pub struct TradingView {
  pub buffer_arc: Arc<Mutex<Vec<Value>>>,
  pub sender: UnboundedSender<Value>,
  pub shutdown: Shutdown,
}

impl TradingView {
  pub async fn new() -> TradingView {
    let rt_handle = tokio::runtime::Handle::current();
    let shutdown = async_shutdown::Shutdown::new();
    // buffer
    let buffer_arc = Arc::new(Mutex::new(vec![]));
    // channel
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Value>();
    // websocket
    let url = String::from("wss://data.tradingview.com/socket.io/websocket");
    let mut ws_builder = websocket_lite::ClientBuilder::new(&url).unwrap();
    ws_builder.add_header(String::from("Origin"), String::from("https://s.tradingview.com"));
    let ws = ws_builder.async_connect().await.unwrap();
    let (mut ws_sink, mut ws_stream) = ws.split::<Message>();
    // mpsc recv -> websocket send thread
    let local_shutdown = shutdown.clone();
    let _mpsc_recv_handle = rt_handle.spawn(async move {
      loop {
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
    let local_shutdown = shutdown.clone();
    let local_buffer_arc = buffer_arc.clone();
    let _ws_recv_handle = rt_handle.spawn(async move {
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
    let local_buffer_arc = buffer_arc.clone();
    return TradingView {
      buffer_arc: local_buffer_arc,
      sender,
      shutdown,
    };
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
}