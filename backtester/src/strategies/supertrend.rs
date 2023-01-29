use serde::Serialize;
use ta::{indicators::Supertrend, DataItem, Next};

use crate::structs::{Candle, Direction, SignalSnapshot};

#[derive(Serialize, Clone, Debug)]
pub struct SupertrendStrategyIndicatorSettings {
  pub supertrend_periods: usize,
  pub supertrend_multiplier: f64,
}

pub struct SupertrendStrategy {}

impl SupertrendStrategy {
  pub fn new() -> SupertrendStrategy {
    return SupertrendStrategy {};
  }

  pub fn build_signal_snapshots_from_candles(&self, indicator_settings: &SupertrendStrategyIndicatorSettings, candles: &Vec<Candle>) -> Vec<SignalSnapshot> {
    // build indicators
    let mut supertrend_indicator = Supertrend::new(indicator_settings.supertrend_periods, indicator_settings.supertrend_multiplier);
    // loop candles
    let mut snapshots: Vec<SignalSnapshot> = vec![];
    for i in 0..candles.len() {
      let candle = &candles[i];
      let open = candle.open;
      let high = candle.high;
      let low = candle.low;
      let close = candle.close;
      let volume = candle.volume as f64;
      let data_item = DataItem::builder().high(high).low(low).close(close).open(open).volume(volume).build().unwrap();
      // supertrend
      let (_supertrend_upper_band, _supertrend_lower_band, supertrend_direction) = supertrend_indicator.next(&data_item);
      snapshots.push(SignalSnapshot {
        candle: candle.clone(),
        direction: if supertrend_direction == -1 { Direction::Short } else { Direction::Long },
      });
    }
    return snapshots;
  }
}
