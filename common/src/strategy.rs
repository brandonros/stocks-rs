use chrono::DateTime;
use chrono_tz::Tz;
use ta::{indicators::SimpleMovingAverage, Next};
use crate::{structs::*, market_session::{self, MarketSessionType}};

pub fn calculate_direction_snapshot(pointer: DateTime<Tz>, reduced_candles: &[Candle], trade_generation_context: &TradeGenerationContext) -> Direction {
  // TODO: not enough candles yet?
  if reduced_candles.len() == 0 {
    return Direction::Flat;
  }
  // do not take trades pre market
  let session_type = market_session::determine_session_type(pointer);
  if session_type != MarketSessionType::Regular {
    return Direction::Flat;
  }
  // two moving averages + crossabove + crossunder
  let mut slow_sma = SimpleMovingAverage::new(trade_generation_context.slow_sma_periods).unwrap();
  let mut fast_sma = SimpleMovingAverage::new(trade_generation_context.fast_sma_periods).unwrap();
  let mut last_fast_sma = 0.0;
  let mut last_slow_sma = 0.0;
  for candle in reduced_candles {
    let hlc3 = (candle.high + candle.low + candle.close) / 3.0;
    last_slow_sma = slow_sma.next(hlc3);
    last_fast_sma = fast_sma.next(hlc3);
  }
  if last_fast_sma > last_slow_sma {
    return Direction::Long;
  }
  return Direction::Short;
}
