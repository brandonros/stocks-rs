pub mod supertrend;
pub mod vwap_mvwap_ema_crossover;

pub use supertrend::*;
pub use vwap_mvwap_ema_crossover::*;

use std::str::FromStr;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum Strategy {
  Supertrend,
  VwapMvwapEmaCrossover,
}

impl FromStr for Strategy {
  type Err = ();

  fn from_str(s: &str) -> Result<Strategy, ()> {
    match s {
      "supertrend" => Ok(Strategy::Supertrend),
      "vwap_mvwap_ema_crossover" => Ok(Strategy::VwapMvwapEmaCrossover),
      _ => Err(()),
    }
  }
}

#[derive(Serialize, Clone, Debug)]
pub enum StrategyIndicatorSettings {
  Supertrend(supertrend::SupertrendStrategyIndicatorSettings),
  VwapMvwapEmaCrossover(vwap_mvwap_ema_crossover::VwapMvwapEmaCrossoverStrategyIndicatorSettings),
}
