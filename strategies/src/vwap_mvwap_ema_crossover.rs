use serde::Serialize;
use ta::{
  indicators::{ExponentialMovingAverage, VolumeWeightedAveragePrice},
  DataItem, Next,
};

use common::structs::*;

#[derive(Serialize, Clone, Debug)]
pub struct VwapMvwapEmaCrossoverStrategyIndicatorSettings {
  pub vwap_ema_fast_periods: usize,
  pub vwap_ema_slow_periods: usize,
  pub ema_fast_periods: usize,
  pub ema_slow_periods: usize,
}

pub struct VwapMvwapEmaCrossoverStrategy {}

impl VwapMvwapEmaCrossoverStrategy {
  pub fn new() -> VwapMvwapEmaCrossoverStrategy {
    return VwapMvwapEmaCrossoverStrategy {};
  }

  pub fn build_signal_snapshots_from_candles(
    &self,
    indicator_settings: &VwapMvwapEmaCrossoverStrategyIndicatorSettings,
    candles: &[Candle],
  ) -> Vec<SignalSnapshot> {
    // build indicators
    let mut vwap_indicator = VolumeWeightedAveragePrice::new();
    let mut vwap_ema_fast_indicator = ExponentialMovingAverage::new(indicator_settings.vwap_ema_fast_periods).unwrap();
    let mut vwap_ema_slow_indicator = ExponentialMovingAverage::new(indicator_settings.vwap_ema_slow_periods).unwrap();
    let mut ema_fast_indicator = ExponentialMovingAverage::new(indicator_settings.ema_fast_periods).unwrap();
    let mut ema_slow_indicator = ExponentialMovingAverage::new(indicator_settings.ema_slow_periods).unwrap();
    // loop candles
    let mut snapshots: Vec<SignalSnapshot> = vec![];
    for candle in candles {
      let open = candle.open;
      let high = candle.high;
      let low = candle.low;
      let close = candle.close;
      let volume = candle.volume as f64;
      let data_item = DataItem::builder().high(high).low(low).close(close).open(open).volume(volume).build().unwrap();
      // vwap
      let vwap = vwap_indicator.next(&data_item);
      // vwap ema fast/slow
      let vwap_ema_fast = vwap_ema_fast_indicator.next(vwap);
      let vwap_ema_slow = vwap_ema_slow_indicator.next(vwap);
      // close ema fast/slow
      let ema_fast = ema_fast_indicator.next(&data_item);
      let ema_slow = ema_slow_indicator.next(&data_item);
      // vwap/mvwap/ema crossover
      let vwap_mvwap_ema_crossover_direction = if vwap_ema_fast >= vwap_ema_slow && ema_fast >= vwap_ema_slow && ema_slow >= vwap_ema_slow {
        Direction::Long
      } else {
        Direction::Short
      };
      snapshots.push(SignalSnapshot {
        candle: candle.clone(),
        direction: vwap_mvwap_ema_crossover_direction,
      });
    }
    return snapshots;
  }
}
