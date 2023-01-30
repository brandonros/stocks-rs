use crate::structs::*;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use rust_decimal_macros::dec;
use strategies::{*, supertrend::*, vwap_mvwap_ema_crossover::*};

fn build_decimal_range(min: Decimal, max: Decimal, step: Decimal) -> Vec<Decimal> {
  let mut pointer = min;
  let mut results = vec![];
  while pointer <= max {
    results.push(pointer);
    pointer += step;
  }
  return results;
}

pub fn build_indicator_setting_combinations(strategy: &Strategy) -> Vec<StrategyIndicatorSettings> {
  let supertrend_periods: Vec<usize> = (5..30).step_by(1).collect();
  let supertrend_multipliers = build_decimal_range(dec!(0.25), dec!(4.0), dec!(0.25));
  let mut combinations = vec![];
  for supertrend_period in &supertrend_periods {
    for supertrend_multiplier in &supertrend_multipliers {
      let indicator_settings = match strategy {
        Strategy::Supertrend => StrategyIndicatorSettings::Supertrend(SupertrendStrategyIndicatorSettings {
          supertrend_periods: *supertrend_period,
          supertrend_multiplier: supertrend_multiplier.to_f64().unwrap(),
        }),
        Strategy::VwapMvwapEmaCrossover => StrategyIndicatorSettings::VwapMvwapEmaCrossover(VwapMvwapEmaCrossoverStrategyIndicatorSettings {
          vwap_ema_fast_periods: 1,
          vwap_ema_slow_periods: 21,
          ema_fast_periods: 7,
          ema_slow_periods: 25,
        }),
      };
      combinations.push(indicator_settings);
    }
  }
  return combinations;
}

pub fn build_backtest_setting_combinations() -> Vec<BacktestSettings> {
  let profit_limit_percentages = build_decimal_range(dec!(0.0005), dec!(0.01), dec!(0.0005));
  let stop_loss_percentages = build_decimal_range(dec!(-0.01), dec!(-0.0005), dec!(0.0005));
  //let stop_loss_percentages = vec![dec!(-1.0)]; // temporarily disable stop losses?
  let mut combinations = vec![];
  for profit_limit_percentage in &profit_limit_percentages {
    for stop_loss_percentage in &stop_loss_percentages {
      let backtest_settings = BacktestSettings {
        slippage_percentage: 0.00025,
        profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
        stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
        warmed_up_index: 10,
        backtest_mode: BacktestMode::SingleEntry,
      };
      combinations.push(backtest_settings);
    }
  }
  return combinations;
}
