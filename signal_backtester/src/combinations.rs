use common::utilities;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

#[derive(Clone, Copy, Debug)]
pub struct BacktestCombination {
  pub supertrend_periods: usize,
  pub supertrend_multiplier: f64,
  pub profit_limit_percentage: f64,
  pub stop_loss_percentage: f64,
  pub warmed_up_index: usize,
}

pub fn build_combinations(mode: &str) -> Vec<BacktestCombination> {
  if mode == "static" {
    return vec![BacktestCombination {
      supertrend_periods: 10,
      supertrend_multiplier: 3.0,
      profit_limit_percentage: 0.005,
      stop_loss_percentage: -0.01,
      warmed_up_index: 0,
    }];
  }
  //let warmed_up_indices: Vec<usize> = (0..30).step_by(1).collect();
  let warmed_up_indices = vec![0]; // TODO
  let supertrend_periods: Vec<usize> = (5..30).step_by(1).collect();
  //let supertrend_multipliers = utilities::build_decimal_range(dec!(0.25), dec!(4.0), dec!(0.25));
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (2.7204, 2056, 19, BacktestCombination { supertrend_periods: 24, supertrend_multiplier: 0.25, profit_limit_percentage: 0.01, stop_loss_percentage: -0.0065, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (2.08698, 1478, 19, BacktestCombination { supertrend_periods: 17, supertrend_multiplier: 0.5, profit_limit_percentage: 0.009, stop_loss_percentage: -0.005, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (1.54963, 1121, 19, BacktestCombination { supertrend_periods: 11, supertrend_multiplier: 0.75, profit_limit_percentage: 0.009, stop_loss_percentage: -0.005, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (1.14809, 850, 19, BacktestCombination { supertrend_periods: 6, supertrend_multiplier: 1.0, profit_limit_percentage: 0.0085, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.83293, 655, 19, BacktestCombination { supertrend_periods: 24, supertrend_multiplier: 1.25, profit_limit_percentage: 0.0085, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.66266, 537, 19, BacktestCombination { supertrend_periods: 20, supertrend_multiplier: 1.5, profit_limit_percentage: 0.01, stop_loss_percentage: -0.0065, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.54982, 446, 19, BacktestCombination { supertrend_periods: 8, supertrend_multiplier: 1.75, profit_limit_percentage: 0.0085, stop_loss_percentage: -0.003, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.4476, 390, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 2.0, profit_limit_percentage: 0.0065, stop_loss_percentage: -0.005, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.36896, 332, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 2.25, profit_limit_percentage: 0.0085, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.32033, 288, 19, BacktestCombination { supertrend_periods: 6, supertrend_multiplier: 2.5, profit_limit_percentage: 0.01, stop_loss_percentage: -0.006, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.28407, 261, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 2.75, profit_limit_percentage: 0.0055, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.26181, 228, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 3.0, profit_limit_percentage: 0.01, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.2358, 210, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 3.25, profit_limit_percentage: 0.0095, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.23484, 180, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 3.5, profit_limit_percentage: 0.0095, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.2107, 164, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 3.75, profit_limit_percentage: 0.0055, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  // 2023-01-01 00:00:00-2023-02-04 00:00:00: (0.19637, 152, 19, BacktestCombination { supertrend_periods: 5, supertrend_multiplier: 4.0, profit_limit_percentage: 0.01, stop_loss_percentage: -0.01, warmed_up_index: 0 })
  let supertrend_multipliers = vec![dec!(1.00)];
  let profit_limit_percentages = utilities::build_decimal_range(dec!(0.0005), dec!(0.01), dec!(0.0005));
  let stop_loss_percentages = utilities::build_decimal_range(dec!(-0.01), dec!(-0.0005), dec!(0.0005)); // TODO
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
              warmed_up_index: *warmed_up_index
            });
          }
        }
      }
    }
  }
  return combinations;
  
}
