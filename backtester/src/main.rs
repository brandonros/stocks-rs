use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use common::backtesting;
use common::backtesting::TradeBacktestResult;
use common::candles;
use common::market_session;
use common::math;
use common::structs::*;
use common::trading;
use common::utilities;
use rayon::prelude::IntoParallelRefIterator;
use rayon::prelude::ParallelIterator;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

fn generate_backtest_context_combinations() -> Vec<BacktestContext> {
  /*let mut combinations = vec![];
  let min = dec!(0.001);
  let max = dec!(0.01);
  let step = dec!(0.0005);
  let profit_limit_percentages = utilities::build_decimal_range(min, max, step);
  let min = dec!(-0.01);
  let max = dec!(-0.001);
  let step = dec!(0.0005);
  let stop_loss_percentages = utilities::build_decimal_range(min, max, step);
  for profit_limit_percentage in &profit_limit_percentages {
    for stop_loss_percentage in &stop_loss_percentages {
      let backtest_context = BacktestContext {
        slippage_percentage: 0.000125,
        profit_limit_percentage: profit_limit_percentage.to_f64().unwrap(),
        stop_loss_percentage: stop_loss_percentage.to_f64().unwrap(),
      };
      combinations.push(backtest_context);
    }
  }
  return combinations;*/
  return vec![
    BacktestContext::default()
  ];
}

fn generate_trade_generation_context_combinations() -> Vec<TradeGenerationContext> {
  /*let mut combinations = vec![];
  let min = 5;
  let max = 50;
  let step = 5;
  let fast_periods = utilities::build_usize_range(min, max, step);
  let min = 5;
  let max = 200;
  let step = 5;
  let slow_periods = utilities::build_usize_range(min, max, step);
  for slow_periods in &slow_periods {
    for fast_periods in &fast_periods {
      // skip where fast is greater than or equal to slow?
      if fast_periods >= slow_periods {
        continue;
      }
      // skip fast + slow being too close?
      let difference = slow_periods - fast_periods;
      if difference < 5 {
        continue;
      }
      let backtest_context = TradeGenerationContext {
        warmup_periods: 1,
        fast_periods: *fast_periods,
        slow_periods: *slow_periods
      };
      combinations.push(backtest_context);
    }
  }
  return combinations;*/
  return vec![
    TradeGenerationContext { fast_periods: 5, slow_periods: 10, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 5, slow_periods: 15, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 5, slow_periods: 20, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 5, slow_periods: 25, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 5, slow_periods: 30, warmup_periods: 1 },
   // TradeGenerationContext { fast_periods: 10, slow_periods: 15, warmup_periods: 1 },
    TradeGenerationContext { fast_periods: 10, slow_periods: 20, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 10, slow_periods: 25, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 15, slow_periods: 20, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 20, slow_periods: 25, warmup_periods: 1 },
    TradeGenerationContext { fast_periods: 20, slow_periods: 30, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 20, slow_periods: 40, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 25, slow_periods: 30, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 25, slow_periods: 35, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 30, slow_periods: 35, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 35, slow_periods: 40, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 35, slow_periods: 50, warmup_periods: 1 },
    TradeGenerationContext { fast_periods: 30, slow_periods: 50, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 40, slow_periods: 45, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 40, slow_periods: 50, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 45, slow_periods: 50, warmup_periods: 1 },
    //TradeGenerationContext { fast_periods: 50, slow_periods: 55, warmup_periods: 1 },
  ];
  /*return vec![
    TradeGenerationContext::default()
  ];*/
}

fn calculate_trade_result_performance(trade_results: &Vec<TradeBacktestResult>) -> (usize, f64, f64) {
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  let mut num_trades = 0;
  let mut simple_profit_loss_percentage = 0.0;
  for trade_result in trade_results {
    simple_profit_loss_percentage += trade_result.profit_loss_percentage;
    balance *= 1.0 + trade_result.profit_loss_percentage;
    num_trades += 1;
  }
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  return (num_trades, simple_profit_loss_percentage, compounded_profit_loss_percentage);
}

fn print_progress(num_tested: usize, num_total: usize, start: Instant) {
  if num_tested % 100 == 0 {
    let elapsed_ms = start.elapsed().as_millis();
    let elapsed_sec = start.elapsed().as_secs();
    let rate_ms = num_tested as f64 / elapsed_ms as f64;
    let rate_sec = rate_ms * 1000.0;
    let num_left = num_total - num_tested;
    let eta_sec = num_left as f64 / rate_sec as f64;
    let percent = (num_tested as f64 / num_total as f64) * 100.0;
    log::info!("{}/{} {:.0}% elapsed {}s eta {:.0}s {:.0}/sec", num_tested, num_total, percent, elapsed_sec, eta_sec, rate_sec)
  }
}

fn backtest_combinations(dates: &Vec<String>, candles: &Vec<Candle>, trade_generation_context_combinations: &Vec<TradeGenerationContext>, backtest_context_combinations: &Vec<BacktestContext>) -> Vec<CombinationBacktestResult> {
  // calculate total number
  let num_combinations = trade_generation_context_combinations.len() * backtest_context_combinations.len();
  log::info!("num_combinations = {}", num_combinations);
  // start
  let start = std::time::Instant::now();
  let combination_results: Vec<CombinationBacktestResult> = vec![];
  let combination_results = Arc::new(Mutex::new(combination_results));
  trade_generation_context_combinations.par_iter().for_each(|trade_generation_context| {
    // build list of trades
    let trades = trading::generate_continuous_trades(&dates, &trade_generation_context, &candles);
    // backtest trades
    backtest_context_combinations.par_iter().for_each(|backtest_context| {
      let trade_results = backtesting::generate_trades_results(backtest_context, &trades, &candles);
      // score trade results
      let (num_trades, simple_profit_loss_percentage, compounded_profit_loss_percentage) = calculate_trade_result_performance(&trade_results);
      // log
      log::trace!("trade_generation_context = {:?} backtest_context = {:?} {:.2}", trade_generation_context, backtest_context, compounded_profit_loss_percentage);
      // push
      let mut combination_results = combination_results.lock().unwrap();
      combination_results.push(CombinationBacktestResult {
        trade_generation_context: trade_generation_context.clone(),
        backtest_context: backtest_context.clone(),
        num_trades,
        simple_profit_loss_percentage,
        compounded_profit_loss_percentage
      });
      print_progress(combination_results.len(), num_combinations, start);
    });
  });
  let mutex = Arc::try_unwrap(combination_results).unwrap();
  let combination_results = mutex.into_inner().unwrap();
  return combination_results;
}

fn build_candles_date_map(provider_name: &str, symbol: &str, resolution: &str, dates: &Vec<String>) -> HashMap<String, Vec<Candle>> {
  let mut candles_date_map = HashMap::new();
  for date in dates {
    //let (start, end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let (start, end) = market_session::get_extended_market_session_start_and_end_from_string(date);
    let start_timestamp = start.timestamp();
    let end_timestamp = end.timestamp();
    // get candles from files
    let (from, to) = common::market_session::get_extended_market_session_start_and_end_from_string(date);
    let provider_candles = providers::get_cached_candles_by_provider_name(provider_name, symbol, "1", from, to).unwrap(); // always pull from database as 1 minute, scale to 5+ later?
    if provider_candles.len() == 0 {
      panic!("no candles for {date}");
    }
    let one_minute_candles: Vec<Candle> = provider_candles
      .iter()
      .map(|candle_snapshot| {
        return Candle {
          timestamp: candle_snapshot.timestamp,
          open: candle_snapshot.open,
          high: candle_snapshot.high,
          low: candle_snapshot.low,
          close: candle_snapshot.close,
          volume: candle_snapshot.volume as i64,
        };
      })
      .collect();
    // scale candles
    let date_candles = candles::convert_timeframe(start_timestamp, end_timestamp, resolution, &one_minute_candles);
    // insert
    candles_date_map.insert(date.clone(), date_candles);
  }
  return candles_date_map;
}

fn main() {
  // init logging
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // parameters
  let args: Vec<String> = std::env::args().collect();
  let provider_name = args.get(1).unwrap();
  let symbol = args.get(2).unwrap();
  let resolution = args.get(3).unwrap();
  let dates_start = format!("{} 00:00:00", args.get(4).unwrap());
  let dates_end = format!("{} 15:59:59", args.get(5).unwrap());
  // build dates
  let dates = common::dates::build_list_of_trading_dates(&dates_start, &dates_end);
  if dates.len() == 0 {
    panic!("no trading dates {dates_start} - {dates_end}");
  }
  // build candles cache map
  let candles_date_map = build_candles_date_map(provider_name, symbol, resolution, &dates);
  let candles = candles::get_candles_by_date_as_continuous_vec(&dates, &candles_date_map);
  log::info!("num_dates = {} num_candles = {}", dates.len(), candles.len());
  if candles.len() == 0 {
    panic!("no candles? {dates_start} - {dates_end}");
  }
  // build list of combinations
  let trade_generation_context_combinations = generate_trade_generation_context_combinations();
  let backtest_context_combinations = generate_backtest_context_combinations();
  // configure thread pool
  rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
  // backtest combinations
  let mut combination_results = backtest_combinations(&dates, &candles, &trade_generation_context_combinations, &backtest_context_combinations);
  // sort by most profitable to least profitable
  combination_results.sort_by(|a, b| {
    let a_score = a.compounded_profit_loss_percentage;
    let b_score = b.compounded_profit_loss_percentage;
    return b_score.partial_cmp(&a_score).unwrap();
  });
  // build aggregatble friendly output
  let mut values = vec![];
  for combination_result in combination_results {
    let trades = trading::generate_continuous_trades(&dates, &combination_result.trade_generation_context, &candles);
    let trade_results = backtesting::generate_trades_results(&combination_result.backtest_context, &trades, &candles);
    let value = serde_json::json!({
      "combination_result": combination_result,
      "trades": trades,
      "trade_results": trade_results,
      "dates_start": dates_start,
      "dates_end": dates_end
    });
    values.push(value);
  }
  let stringified_value = serde_json::to_string_pretty(&values).unwrap();
  let mut file = std::fs::File::create(format!("./output/{dates_start}-{dates_end}.json")).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
}
