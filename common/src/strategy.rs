use chrono::DateTime;
use chrono_tz::Tz;
use ta::{indicators::SimpleMovingAverage, Next};
use crate::{structs::*, market_session::{self, MarketSessionType}};

pub fn calculate_direction_snapshot(_start: DateTime<Tz>, end: DateTime<Tz>, pointer: DateTime<Tz>, reduced_candles: &[Candle], trade_generation_context: &TradeGenerationContext) -> Direction {
  // TODO: not enough candles yet?
  if reduced_candles.len() == 0 {
    return Direction::Flat;
  }
  // do not take trades pre market
  let session_type = market_session::determine_session_type(pointer);
  if session_type != MarketSessionType::Regular {
    return Direction::Flat;
  }
  // go flat 5 minutes (1 candle) before close?
  let distance_to_close = (end - pointer).num_seconds();
  if distance_to_close <= 300 - 1 { // end is 15:59:59 and not 16:00:00
    return Direction::Flat;
  }
  // two moving averages + crossabove + crossunder
  let mut slow = SimpleMovingAverage::new(trade_generation_context.slow_periods).unwrap();
  let mut fast = SimpleMovingAverage::new(trade_generation_context.fast_periods).unwrap();
  let mut last_fast = 0.0;
  let mut last_slow = 0.0;
  for candle in reduced_candles {
    let hlc3 = (candle.high + candle.low + candle.close) / 3.0;
    last_slow = slow.next(hlc3);
    last_fast = fast.next(hlc3);
  }
  if last_fast > last_slow {
    return Direction::Long;
  }
  return Direction::Short;
}
