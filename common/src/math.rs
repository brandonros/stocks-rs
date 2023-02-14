use super::structs::*;

#[allow(dead_code)]
pub fn round(x: f64, decimals: u32) -> f64 {
  let y = 10_i64.pow(decimals) as f64;
  return (x * y).round() / y;
}

pub fn calculate_profit_loss(trade_direction: &Direction, open_price: f64, exit_price: f64) -> f64 {
  if *trade_direction == Direction::Long {
    return exit_price - open_price; // TODO: slippage?
  }
  return open_price - exit_price;
}

pub fn calculate_percentage_increase(old_value: f64, new_value: f64) -> f64 {
  let delta = new_value - old_value;
  return delta / old_value;
}

pub fn calculate_percentage_decrease(old_value: f64, new_value: f64) -> f64 {
  let delta = old_value - new_value;
  return delta / old_value;
}

pub fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
  if old_value <= new_value {
    return calculate_percentage_increase(old_value, new_value);
  }
  return calculate_percentage_decrease(old_value, new_value);
}

pub fn calculate_open_price_with_slippage(trade_direction: &Direction, open_price: f64, slippage_percentage: f64) -> f64 {
  let slippage = open_price * slippage_percentage;
  if *trade_direction == Direction::Long {
    return open_price + slippage;
  }
  return open_price - slippage;
}

pub fn calculate_close_price_with_slippage(trade_direction: &Direction, close_price: f64, slippage_percentage: f64) -> f64 {
  let slippage = close_price * slippage_percentage;
  if *trade_direction == Direction::Long {
    return close_price - slippage;
  }
  return close_price + slippage;
}

pub fn calculate_profit_limit_price(trade_direction: &Direction, open_price: f64, profit_limit_percentage: f64) -> f64 {
  if *trade_direction == Direction::Long {
    let profit_limit_price = open_price * (1.0 + profit_limit_percentage);
    return profit_limit_price;
  }
  let profit_limit_price = open_price * (1.0 - profit_limit_percentage);
  return profit_limit_price;
}

pub fn calculate_stop_loss_price(trade_direction: &Direction, open_price: f64, stop_loss_percentage: f64) -> f64 {
  if *trade_direction == Direction::Long {
    let stop_loss_price = open_price * (1.0 - stop_loss_percentage.abs());
    return stop_loss_price;
  }
  let stop_loss_price = open_price * (1.0 + stop_loss_percentage.abs());
  return stop_loss_price;
}

pub fn calculate_profit_loss_percentage(trade_direction: &Direction, open_price: f64, exit_price: f64) -> f64 {
  if *trade_direction == Direction::Long {
    return calculate_percentage_increase(open_price, exit_price);
  }
  return calculate_percentage_decrease(open_price, exit_price);
}

pub fn calculate_best_case_scenario_price(trade_direction: &Direction, candle: &Candle) -> f64 {
  if *trade_direction == Direction::Long {
    return candle.high;
  }
  return candle.low;
}

pub fn calculate_worst_case_scenario_price(trade_direction: &Direction, candle: &Candle) -> f64 {
  if *trade_direction == Direction::Long {
    return candle.low;
  }
  return candle.high;
}

pub fn normalize(x: f64, min: f64, max: f64) -> f64 {
  if min == max {
    return 0.0;
  }
  return (x - min) / (max - min);
}
