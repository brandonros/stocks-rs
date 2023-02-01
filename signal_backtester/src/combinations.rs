#[derive(Debug)]
pub struct BacktestCombination {
  pub supertrend_periods: usize,
  pub supertrend_multiplier: f64,
  pub profit_limit_percentage: f64,
  pub stop_loss_percentage: f64,
  pub warmed_up_index: usize
}

pub fn build_combinations() -> Vec<BacktestCombination> {
  /*//let warmed_up_indices: Vec<usize> = (0..30).step_by(1).collect();
  let warmed_up_indices = vec![0]; // TODO
  let supertrend_periods: Vec<usize> = (5..30).step_by(1).collect();
  let supertrend_multipliers = utilities::build_decimal_range(dec!(0.25), dec!(4.0), dec!(0.25));
  let profit_limit_percentages = utilities::build_decimal_range(dec!(0.0005), dec!(0.01), dec!(0.0005));
  //let stop_loss_percentages = utilities::build_decimal_range(dec!(-0.01), dec!(-0.0005), dec!(0.0005)); // TODO
  let stop_loss_percentages = vec![dec!(-1.00)];
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
  return combinations;*/
  return vec![
    BacktestCombination {
      supertrend_periods: 10,
      supertrend_multiplier: 3.0,
      profit_limit_percentage: 0.005,
      stop_loss_percentage: -0.01,
      warmed_up_index: 0,
    }
  ];
}
