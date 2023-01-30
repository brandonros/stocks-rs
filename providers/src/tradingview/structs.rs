use chrono::NaiveDateTime;
use common::json_time;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Clone, Eq, PartialEq, Debug)]
pub enum TradingViewMessageType {
  ServerInfo,
  Ping,
  QsdFounded,
  QsdSessionHolidays,
  QsdBidAsk,
  QsdLastPrice,
  QsdVolume,
  QsdRtUpdateTime,
  QsdFundamentalData,
  QsdEmptyTradeLoaded,
  QsdEmptyRtcTime,
  QsdAfterMarketLastPrice,
  DuSeries,
  DuStudy,
  TimescaleUpdate,
  StudyCompleted,
  SeriesCompleted,
  SymbolResolved,
  QuoteCompleted,
  SeriesLoading,
  StudyLoading,
}

#[derive(Serialize, Clone, Debug)]
pub struct TradingViewMessage {
  pub timestamp: i64,
  pub message_type: TradingViewMessageType,
  pub value: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct IndicatorSnapshot {
  pub source: String,
  pub symbol: String,
  pub indicator_name: String,
  pub timeframe: String,
  pub session_type: String,
  #[serde(with = "json_time")]
  pub candle_timestamp: NaiveDateTime,
  #[serde(with = "json_time")]
  pub scraped_at: NaiveDateTime,
  pub direction: String,
  pub underlying_mark_price: f64,
  pub underlying_last_price: f64,
}
