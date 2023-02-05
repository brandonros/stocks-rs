mod combinations;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono_tz::US::Eastern;
use combinations::BacktestCombination;
use common::database::Database;
use common::{structs::*, math};
use common::{database};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use strategies::supertrend::{SupertrendStrategy, SupertrendStrategyIndicatorSettings};

fn get_candle_snapshots_from_database(connection: &Database, symbol: &str, resolution: &str, eastern_now_timestamp: i64, regular_market_start_timestamp: i64, candle_lookup_max_timestamp: i64) -> Vec<CandleSnapshot> {
  // TODO: put this back? it doesn't work well for historical and scraped_at = (select scraped_at from candles where scraped_at >= {regular_market_start_timestamp} and scraped_at <= {eastern_now_timestamp} order by scraped_at desc limit 1) 
  let query = format!(
    "
    select scraped_at,
      timestamp, 
      open, 
      high, 
      low,
      close,
      volume
    from candles 
    where timestamp >= {regular_market_start_timestamp} and timestamp <= {candle_lookup_max_timestamp}
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
    "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

fn determine_trade_result(slippage_percentage: f64, profit_limit_percentage: f64, stop_loss_percentage: f64, trade_signal_snapshots: &[SignalSnapshot]) -> ReducedBacktestResult {
  let start_signal_snapshot = &trade_signal_snapshots[0];
  let end_signal_snapshot = &trade_signal_snapshots[trade_signal_snapshots.len() - 1];
  // assume we put an order in to open a position at the start of new candle (open price) and get filled with some slippage
  let open_price = math::calculate_open_price_with_slippage(start_signal_snapshot.direction, start_signal_snapshot.candle.open, slippage_percentage);
  // calculate profit limit + stop loss based on fill price without any exit slippage?
  let profit_limit_price = math::calculate_profit_limit_price(start_signal_snapshot.direction, open_price, profit_limit_percentage);
  let stop_loss_price = math::calculate_stop_loss_price(start_signal_snapshot.direction, open_price, stop_loss_percentage);
  for trade_signal_snapshot in trade_signal_snapshots {
    // calculate worst/best based on direction
    // TODO: include slippage here or on the exit or assume exact exit price?
    let hypothetical_best_case_scenario_price = math::calculate_best_case_scenario_price(start_signal_snapshot.direction, &trade_signal_snapshot.candle);
    let hypothetical_worst_case_scenario_price = math::calculate_worst_case_scenario_price(start_signal_snapshot.direction, &trade_signal_snapshot.candle);
    let hypothetical_best_case_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, open_price, hypothetical_best_case_scenario_price);
    let hypothetical_worst_case_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, open_price, hypothetical_worst_case_scenario_price);
    // always check stop loss first to be as pessimistic as possible because since the candle is rolled up 60 seconds into 1 minute we don't know when we actually exit/if we hit the low before the high or not
    if hypothetical_worst_case_profit_loss_percentage <= stop_loss_percentage {
      let exit_price = stop_loss_price; // TODO: add slippage to exit? we would have opened a profit limit and a stop loss (or stop limit?) order at the time of open/fill
      let profit_loss = math::calculate_profit_loss(start_signal_snapshot.direction, open_price, exit_price);
      return ReducedBacktestResult {
        open_price: math::round(open_price, 2),
        exit_price: math::round(exit_price, 2),
        profit_limit_price: math::round(profit_limit_price, 2),
        stop_loss_price: math::round(stop_loss_price, 2),
        outcome: BacktestOutcome::StopLoss,
        trade_entry_snapshot: start_signal_snapshot.clone(),
        trade_exit_snapshot: trade_signal_snapshot.clone(),
        trade_duration: trade_signal_snapshot.candle.timestamp - start_signal_snapshot.candle.timestamp,
        profit_loss: math::round(profit_loss, 2),
        profit_loss_percentage: math::round(stop_loss_percentage, 5) // assume no better or worse than stop loss percentage exactly?
      };
    }
    if hypothetical_best_case_profit_loss_percentage >= profit_limit_percentage {
      let exit_price = profit_limit_price; // TODO: add slippage to exit? we would have opened a profit limit and a stop loss (or stop limit?) order at the time of open/fill
      let profit_loss = math::calculate_profit_loss(start_signal_snapshot.direction, open_price, exit_price);
      return ReducedBacktestResult {
        open_price: math::round(open_price, 2),
        exit_price: math::round(exit_price, 2),
        profit_limit_price: math::round(profit_limit_price, 2),
        stop_loss_price: math::round(stop_loss_price, 2),
        outcome: BacktestOutcome::ProfitLimit,
        trade_entry_snapshot: start_signal_snapshot.clone(),
        trade_exit_snapshot: trade_signal_snapshot.clone(),
        trade_duration: trade_signal_snapshot.candle.timestamp - start_signal_snapshot.candle.timestamp,
        profit_loss: math::round(profit_loss, 2),
        profit_loss_percentage: math::round(profit_limit_percentage, 5) // assume no better or worse than profit limit percentage exactly?
      };
    }
  }
  let exit_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, end_signal_snapshot.candle.close, slippage_percentage); // TODO: would probably get out on next candle open instead of last candle close? but we don't include this next candle on purpose? / add slippage to exit?
  let exit_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, open_price, exit_price);
  let profit_loss = math::calculate_profit_loss(start_signal_snapshot.direction, open_price, exit_price);
  return ReducedBacktestResult {
    open_price: math::round(open_price, 2),
    exit_price: math::round(exit_price, 2),
    profit_limit_price: math::round(profit_limit_price, 2),
    stop_loss_price: math::round(stop_loss_price, 2),
    outcome: BacktestOutcome::DirectionChange,
    trade_entry_snapshot: start_signal_snapshot.clone(),
    trade_exit_snapshot: end_signal_snapshot.clone(),
    trade_duration: end_signal_snapshot.candle.timestamp - start_signal_snapshot.candle.timestamp,
    profit_loss: math::round(profit_loss, 2),
    profit_loss_percentage: math::round(exit_profit_loss_percentage, 5) // assume no better or worse than profit limit percentage exactly?
  };
}

fn backtest_date(candles_map: &HashMap<String, Vec<CandleSnapshot>>, symbol: &str, resolution: &str, warmed_up_index: usize, indicator_settings: &SupertrendStrategyIndicatorSettings, slippage_percentage: f64, profit_limit_percentage: f64, stop_loss_percentage: f64, date: &str) -> Result<Vec<ReducedBacktestResult>, String> {
  let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
  // get mock now from end of day TODO: no need to walk through entire day second by second/minute by minute? use end of day direction changes as "collective" answer to what happened throughout the day
  let eastern_now = regular_market_end.with_timezone(&Eastern);
  let eastern_now_timestamp = eastern_now.timestamp();
  let regular_market_start_timestamp = regular_market_start.timestamp();
  let (current_candle_start, _current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, &eastern_now);
  // get candles from database
  // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
  //let candle_lookup_max_timestamp = eastern_now_timestamp;
  let candle_lookup_max_timestamp = current_candle_start.timestamp() - 1;
  let candle_snapshots = candles_map.get(date).unwrap();
  if candle_snapshots.len() == 0 {
    return Err("candles.len() == 0".to_string());    
  }
  // convert candle snapshots to candles
  let candles = candle_snapshots.into_iter().map(|candle_snapshot| {
    return Candle {
      timestamp: candle_snapshot.timestamp,
      open: candle_snapshot.open,
      high: candle_snapshot.high,
      low: candle_snapshot.low,
      close: candle_snapshot.close,
      volume: candle_snapshot.volume,
    };
  }).collect();
  // get recent signal signal from candles
  let strategy = SupertrendStrategy::new();
  let signal_snapshots = strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles);
  if signal_snapshots.is_empty() {
    return Err("signal_snapshots.len() == 0".to_string());    
  }
  // check snapshot age?
  let most_recent_signal_snapshot = &signal_snapshots[signal_snapshots.len() - 1];
  let most_recent_signal_snapshot_candle_age = eastern_now_timestamp - most_recent_signal_snapshot.candle.timestamp;
  if most_recent_signal_snapshot_candle_age > 120 {
    log::warn!("signal_snapshot candle is old! most_recent_signal_snapshot_candle_age = {}", most_recent_signal_snapshot_candle_age);
  }
  // get direction changes
  let direction_changes = strategies::build_direction_changes_from_signal_snapshots(&signal_snapshots, warmed_up_index);
  if direction_changes.is_empty() {
    return Err("direction_changes.len() == 0".to_string());    
  }
  // backtest direction changes as trades
  let mut results = vec![];
  for direction_change in &direction_changes {
    let start_snapshot_index = direction_change.start_snapshot_index;
    let end_snapshot_index = direction_change.end_snapshot_index.unwrap();
    let trade_signal_snapshots = &signal_snapshots[start_snapshot_index..=end_snapshot_index];
    let trade_result = determine_trade_result(slippage_percentage, profit_limit_percentage, stop_loss_percentage, trade_signal_snapshots);
    results.push(trade_result);
  }
  return Ok(results);
}

fn main() {
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let args: Vec<String> = std::env::args().collect();
    let provider_name = args.get(1).unwrap();
    let _strategy_name = args.get(2).unwrap();
    let symbol = args.get(3).unwrap();
    let resolution = args.get(4).unwrap();
    let start_date = format!("{} 00:00:00", args.get(5).unwrap());
    let end_date = format!("{} 00:00:00", args.get(6).unwrap());   
    let dates = common::dates::build_list_of_dates(&start_date, &end_date);
    // open database
    let database_filename = format!("./database-{}.db", provider_name);
    let connection = database::Database::new(&database_filename);
    // build cache
    let mut candles_map = HashMap::new();
    for date in &dates {
      let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
      let eastern_now = regular_market_end.with_timezone(&Eastern);
      let eastern_now_timestamp = eastern_now.timestamp();
      let regular_market_start_timestamp = regular_market_start.timestamp();
      let (current_candle_start, _current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, &eastern_now);
      // get candles from database
      // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
      //let candle_lookup_max_timestamp = eastern_now_timestamp;
      let candle_lookup_max_timestamp = current_candle_start.timestamp() - 1;
      let candle_snapshots = get_candle_snapshots_from_database(&connection, symbol, resolution, eastern_now_timestamp, regular_market_start_timestamp, candle_lookup_max_timestamp);
      candles_map.insert(date.clone(), candle_snapshots);
    }
    // build combinations
    let combinations = combinations::build_combinations("cartesian");
    let num_combinations = combinations.len();
    log::info!("{} combinations", num_combinations);
    // init database tables
    connection.migrate("./schema/");
    // backtest
    let start = std::time::Instant::now();
    let backtest_results: Vec<(f64, usize, usize, BacktestCombination)> = vec![];
    let backtest_results = Arc::new(Mutex::new(backtest_results));
    combinations.into_par_iter().for_each(|combination| {
      let indicator_settings = SupertrendStrategyIndicatorSettings {
        supertrend_periods: combination.supertrend_periods,
        supertrend_multiplier: combination.supertrend_multiplier,
      };
      let warmed_up_index = combination.supertrend_periods; // TODO: does this need to be indicator_settings.supertrend_periods 1:1 to make sure the moving average gets warmed up?
      let slippage_percentage = 0.000125; // about $0.05 on a $400 share
      let profit_limit_percentage = combination.profit_limit_percentage;
      let stop_loss_percentage = combination.stop_loss_percentage;
      let starting_balance = 1000.00;
      let mut balance = starting_balance;
      let mut num_trades = 0;
      let mut num_days = 0;
      for date in &dates {
        let results = backtest_date(&candles_map, symbol, resolution, warmed_up_index, &indicator_settings, slippage_percentage, profit_limit_percentage, stop_loss_percentage, date);
        if results.is_err() {
          //log::error!("{} {:?}", date, results.err());
          continue;
        }
        num_days += 1;
        let results = results.unwrap();
        for result in &results {
          balance *= 1.0 + result.profit_loss_percentage;
        }
        num_trades += results.len();
      }
      let profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
      let profit_loss_percentage = math::round(profit_loss_percentage, 5);
      let mut backtest_results = backtest_results.lock().unwrap();
      backtest_results.push((profit_loss_percentage, num_trades, num_days, combination.clone()));
      let num_tested = backtest_results.len();
      drop(backtest_results);
      // print
      if num_tested % 1000 == 0 {
        let elapsed = start.elapsed().as_millis();
        let rate = (num_tested as f64 / elapsed as f64) * 1000.0;
        let num_left = num_combinations - num_tested;
        let time_left = num_left as f64 / rate as f64;
        log::info!("{}/{} elapsed: {:.0}s rate: {:.0}/sec eta: {:.0}s", num_tested, num_combinations, elapsed as f64 / 1000.0, rate, time_left);
      }
    });
    let mut backtest_results = backtest_results.lock().unwrap();
    // sort
    backtest_results.sort_by(|a, b| {
      let a_profit_loss_percentage = a.0;
      let b_profit_loss_percentage = b.0;
      return b_profit_loss_percentage.partial_cmp(&a_profit_loss_percentage).unwrap();
    });
    // print best result
    let best_result = &backtest_results[0];
    log::info!("{}-{}: {:?}", start_date, end_date, best_result)
    
  });
}
