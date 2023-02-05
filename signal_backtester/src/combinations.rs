use common::structs::*;
use common::utilities;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

pub fn build_combinations(mode: &str) -> Vec<BacktestCombination> {
  if mode == "static" {
    return vec![BacktestCombination {
      supertrend_periods: 5,
      supertrend_multiplier: 0.5,
      profit_limit_percentage: 0.002,
      stop_loss_percentage: -0.01,
      warmed_up_index: 5,
    }];
  } else if mode == "cartesian" {
    //let warmed_up_indices: Vec<usize> = (0..30).step_by(1).collect();
    let warmed_up_indices = vec![0]; // TODO
    let supertrend_periods: Vec<usize> = (5..30).step_by(1).collect();
    let supertrend_multipliers = utilities::build_decimal_range(dec!(0.50), dec!(4.0), dec!(0.25));
    //let supertrend_multipliers = vec![dec!(2.00)];
    let profit_limit_percentages = utilities::build_decimal_range(dec!(0.001), dec!(0.01), dec!(0.0005));
    //let profit_limit_percentages = utilities::build_decimal_range(dec!(0.0005), dec!(0.0025), dec!(0.0005));
    let stop_loss_percentages = utilities::build_decimal_range(dec!(-0.01), dec!(-0.001), dec!(0.0005));
    //let stop_loss_percentages = utilities::build_decimal_range(dec!(-0.00125), dec!(-0.0005), dec!(0.0005));
    //let stop_loss_percentages = vec![dec!(-1.00)];
    let mut combinations = vec![];
    for warmed_up_index in &warmed_up_indices {
      for profit_limit_percentage in &profit_limit_percentages {
        for stop_loss_percentage in &stop_loss_percentages {
          for supertrend_periods in &supertrend_periods {
            for supertrend_multiplier in &supertrend_multipliers {
              combinations.push(BacktestCombination {
                supertrend_periods: *supertrend_periods,
                supertrend_multiplier: supertrend_multiplier.to_f64().unwrap(),
                profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
                stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
                warmed_up_index: *supertrend_periods, // TODO: *warmed_up_index or *supertrend_periods or 0 or a constant
              });
            }
          }
        }
      }
    }
    return combinations;
  } else {
    unimplemented!();
  }
}
