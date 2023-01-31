pub mod supertrend;
pub mod vwap_mvwap_ema_crossover;

use std::str::FromStr;

use common::structs::*;
use serde::Serialize;
use supertrend::*;
use vwap_mvwap_ema_crossover::*;

#[derive(Serialize, Debug)]
pub enum Strategy {
  Supertrend,
  VwapMvwapEmaCrossover,
}

#[derive(Serialize, Clone, Debug)]
pub enum StrategyIndicatorSettings {
  Supertrend(supertrend::SupertrendStrategyIndicatorSettings),
  VwapMvwapEmaCrossover(vwap_mvwap_ema_crossover::VwapMvwapEmaCrossoverStrategyIndicatorSettings),
}

#[derive(Serialize, Clone, Debug)]
pub struct DirectionChange {
  pub start_snapshot_index: usize,
  pub end_snapshot_index: Option<usize>,
}

impl FromStr for Strategy {
  type Err = ();

  fn from_str(s: &str) -> Result<Strategy, ()> {
    match s {
      "supertrend" => return Ok(Strategy::Supertrend),
      "vwap_mvwap_ema_crossover" => return Ok(Strategy::VwapMvwapEmaCrossover),
      _ => return Err(()),
    }
  }
}

pub fn build_signal_snapshots_from_candles(strategy: &Strategy, indicator_settings: &StrategyIndicatorSettings, candles: &Vec<Candle>) -> Vec<SignalSnapshot> {
  let signal_snapshots = match strategy {
    Strategy::Supertrend => {
      let strategy = SupertrendStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::Supertrend(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(indicator_settings, candles)
    }
    Strategy::VwapMvwapEmaCrossover => {
      let strategy = VwapMvwapEmaCrossoverStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::VwapMvwapEmaCrossover(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(indicator_settings, candles)
    }
  };
  return signal_snapshots;
}

pub fn build_direction_changes_from_signal_snapshots(signal_snapshots: &Vec<SignalSnapshot>, warmed_up_index: usize) -> Vec<DirectionChange> {
  let mut trade_direction = Direction::Flat;
  let mut direction_changes: Vec<DirectionChange> = vec![];
  for (i, signal_snapshot) in signal_snapshots.iter().enumerate().skip(warmed_up_index) {
    let current_direction = signal_snapshot.direction.to_owned();
    if current_direction != trade_direction {
      // close any open trades
      if !direction_changes.is_empty() {
        let last_direction_change_index = direction_changes.len() - 1;
        let mut last_direction_change = &mut direction_changes[last_direction_change_index];
        last_direction_change.end_snapshot_index = Some(i);
      }
      // open new trade
      direction_changes.push(DirectionChange {
        start_snapshot_index: i,
        end_snapshot_index: None,
      });
      trade_direction = current_direction;
    }
  }
  // make sure last trade is closed
  if direction_changes.len() != 0 {
    let last_direction_change_index = direction_changes.len() - 1;
    let mut last_direction_change = &mut direction_changes[last_direction_change_index];
    last_direction_change.end_snapshot_index = Some(signal_snapshots.len() - 1);
  }
  // return
  return direction_changes;
}
