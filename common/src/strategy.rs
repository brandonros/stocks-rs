use chrono::DateTime;
use chrono_tz::Tz;
use crate::structs::*;

pub fn calculate_direction_snapshot(pointer: DateTime<Tz>, reduced_candles: &[Candle], trade_generation_context: &TradeGenerationContext) -> Direction {
  log::info!("{}", pointer.timestamp());
  panic!("TODO");
}
