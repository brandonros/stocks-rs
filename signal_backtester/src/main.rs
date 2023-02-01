mod combinations;

use std::collections::HashMap;

use combinations::BacktestCombination;
use common::database::*;
use common::math;
use common::structs::*;
use common::utilities;
use common::file;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use strategies::supertrend::*;

fn get_candles_from_database(connection: &Database, symbol: &str, resolution: &str, start_timestamp: i64, end_timestamp: i64) -> Vec<Candle> {
  let candles_query = format!(
    "
    with most_recent_candle_snapshots as (
      select max(scraped_at) as scraped_at, symbol, resolution, timestamp from candles
      where scraped_at >= {start_timestamp} and scraped_at <= {end_timestamp} and symbol = '{symbol}' and resolution = '{resolution}'
      group by symbol, resolution, timestamp
    )
    select
      candles.symbol,
      candles.resolution,
      candles.timestamp,
      open,
      high,
      low,
      close,
      volume
    from most_recent_candle_snapshots
    join candles on most_recent_candle_snapshots.scraped_at = candles.scraped_at and 
      most_recent_candle_snapshots.timestamp = candles.timestamp and 
      most_recent_candle_snapshots.symbol = candles.symbol and 
      most_recent_candle_snapshots.resolution = candles.resolution
      where candles.timestamp >= {start_timestamp}
      and candles.timestamp <= {end_timestamp}
      and candles.resolution = '{resolution}'
      and candles.symbol = '{symbol}'
    ORDER BY candles.timestamp ASC
  "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  let candles = connection.get_rows_from_database::<Candle>(&candles_query);
  return candles;
}

fn get_quote_snapshots_from_database(connection: &Database, symbol: &str, start_timestamp: i64, end_timestamp: i64) -> Vec<QuoteSnapshot> {
  let quotes_query = format!(
    "
    select symbol, scraped_at, ask_price, bid_price, last_trade_price
    from quote_snapshots
    where symbol = '{symbol}' and scraped_at >= {start_timestamp} and scraped_at <= {end_timestamp}
    order by scraped_at desc
    limit 1
    "
  );
  let quote_snapshots = connection.get_rows_from_database::<QuoteSnapshot>(&quotes_query);
  return quote_snapshots;
}

fn backtest_combination(candles_timestamp_cache_map: &HashMap::<i64, Vec<Candle>>, quote_snapshots_timestamp_cache_map: &HashMap::<i64, QuoteSnapshot>, date: &str, combination: &BacktestCombination) -> (usize, f64) {
  // config
  let slippage_percentage = 0.000125; // about $0.05 on a $400 share price
  let indicator_settings = SupertrendStrategyIndicatorSettings {
    supertrend_periods: combination.supertrend_periods,
    supertrend_multiplier: combination.supertrend_multiplier,
  };
  let warmed_up_index = combination.warmed_up_index;
  let profit_limit_percentage = combination.profit_limit_percentage;
  let stop_loss_percentage = combination.stop_loss_percentage;
  // times
  let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
  // state
  let mut last_trade_direction = Direction::Flat;
  let mut last_trade_open_quote: Option<QuoteSnapshot> = None;
  let mut is_trade_open = false;
  let mut total_profit_loss_percentage = 0.0;
  let mut num_trades = 0;
  // loop over entire day
  let pointer_increment_seconds = 1;
  let mut pointer = regular_market_start;
  while pointer <= regular_market_end {
    // get timestamps
    let eastern_now = &pointer;
    let eastern_now_timestamp = eastern_now.timestamp();
    // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
    // let (current_candle_start, current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, eastern_now);
    // let candle_lookup_max_timestamp = current_candle_start_timestamp - 1;
    // let expected_max_signal_snapshot_age = 120;
    let candle_lookup_max_timestamp = eastern_now_timestamp;
    let expected_max_signal_snapshot_age = 65;
    // get candles
    let candles = candles_timestamp_cache_map.get(&candle_lookup_max_timestamp).unwrap();
    if candles.len() == 0 {
      log::warn!("{eastern_now_timestamp}: candles.len() == 0");
      pointer += chrono::Duration::seconds(pointer_increment_seconds);
      continue;
    }
    // get most recent signal signal from candles
    let strategy = SupertrendStrategy::new();
    let signal_snapshots = strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles);
    if signal_snapshots.is_empty() {
      log::warn!("{eastern_now_timestamp}: signal_snapshots.len() == 0");
      pointer += chrono::Duration::seconds(pointer_increment_seconds);
      continue;
    }
    // get direction changes from signal snapshots
    let direction_changes = strategies::build_direction_changes_from_signal_snapshots(&signal_snapshots, warmed_up_index);
    if direction_changes.is_empty() {
      log::warn!("{eastern_now_timestamp}: direction_changes.len() == 0");
      pointer += chrono::Duration::seconds(pointer_increment_seconds);
      continue;
    }
    let most_recent_direction_change = &direction_changes[direction_changes.len() - 1];
    let most_recent_direction_change_start_snapshot = &signal_snapshots[most_recent_direction_change.start_snapshot_index];
    // get current quote
    let most_recent_quote_snapshot = quote_snapshots_timestamp_cache_map.get(&eastern_now_timestamp);
    if most_recent_quote_snapshot.is_none() {
      log::warn!("{eastern_now_timestamp}: most_recent_quote_snapshot.is_none()");
      pointer += chrono::Duration::seconds(pointer_increment_seconds);
      continue;
    }
    let most_recent_quote_snapshot = most_recent_quote_snapshot.unwrap();
    // check quote age
    let quote_age = eastern_now_timestamp - most_recent_quote_snapshot.scraped_at;
    // TODO: handle if quote_snapshot is too old/unrealistic from something like a quote_scraper process crash
    if quote_age > 1 {
      log::warn!("{eastern_now_timestamp}: quote_snapshot is old! quote_age = {quote_age}");
    }
    // check snapshot age?
    let most_recent_signal_snapshot = &signal_snapshots[signal_snapshots.len() - 1];
    let signal_snapshot_age = eastern_now_timestamp - most_recent_signal_snapshot.candle.timestamp;
    if signal_snapshot_age > expected_max_signal_snapshot_age {
      log::warn!("{eastern_now_timestamp}: signal_snapshot is old! signal_snapshot_age = {signal_snapshot_age}");
    }
    // check if open trade profit limited/stop lossed
    if is_trade_open {
      let hypothetcial_open_price = math::calculate_open_price_with_slippage(last_trade_direction, last_trade_open_quote.as_ref().unwrap().last_trade_price, slippage_percentage);
      let hypothetical_exit_price = math::calculate_close_price_with_slippage(last_trade_direction, most_recent_quote_snapshot.last_trade_price, slippage_percentage);
      let open_profit_loss_percentage = math::calculate_profit_loss_percentage(last_trade_direction, hypothetcial_open_price, hypothetical_exit_price);
      if open_profit_loss_percentage <= stop_loss_percentage {
        log::info!("csv: close,{eastern_now_timestamp},{:?},{hypothetical_exit_price},StopLoss", last_trade_direction);
        log::info!("{eastern_now_timestamp}: closing trade; stop loss hit; open_profit_loss_percentage = {open_profit_loss_percentage} quote_age = {quote_age}s current_quote = {:?} last_trade_open_quote = {:?}",
          most_recent_quote_snapshot,
          last_trade_open_quote
        );
        // mark trade closed
        total_profit_loss_percentage += stop_loss_percentage;
        num_trades += 1;
        is_trade_open = false;
        last_trade_open_quote = None;
      } else if open_profit_loss_percentage >= profit_limit_percentage {
        log::info!("csv: close,{eastern_now_timestamp},{:?},{hypothetical_exit_price},ProfitLimit", last_trade_direction);
        log::info!("{eastern_now_timestamp}: closing trade; profit limit hit; open_profit_loss_percentage = {open_profit_loss_percentage} quote_age = {quote_age}s current_quote = {:?} last_trade_open_quote = {:?}",
          most_recent_quote_snapshot,
          last_trade_open_quote
        );
        // mark trade closed
        total_profit_loss_percentage += profit_limit_percentage;
        num_trades += 1;
        is_trade_open = false;
        last_trade_open_quote = None;
      }
    }
    // check for direction change
    if last_trade_direction != most_recent_direction_change_start_snapshot.direction {
      let new_direction = most_recent_direction_change_start_snapshot.direction;
      // close any open trades
      if is_trade_open == true {
        let old_direction = last_trade_direction;
        let hypothetcial_open_price = math::calculate_open_price_with_slippage(old_direction, last_trade_open_quote.as_ref().unwrap().last_trade_price, slippage_percentage);
        let hypothetical_exit_price = math::calculate_close_price_with_slippage(old_direction, most_recent_quote_snapshot.last_trade_price, slippage_percentage);
        let open_profit_loss_percentage = math::calculate_profit_loss_percentage(old_direction, hypothetcial_open_price, hypothetical_exit_price);
        log::info!("csv: close,{eastern_now_timestamp},{:?},{hypothetical_exit_price},DirectionChange", old_direction);
        log::info!(
          "{eastern_now_timestamp}: closing trade; direction change; open_profit_loss_percentage = {open_profit_loss_percentage} quote_age = {quote_age}s current_quote = {:?} last_trade_open_quote = {:?}",
          most_recent_quote_snapshot,
          last_trade_open_quote
        );
        // mark trade closed
        total_profit_loss_percentage += open_profit_loss_percentage;
        num_trades += 1;
        last_trade_open_quote = None;
      }
      // open new trade
      let hypothetcial_open_price = math::calculate_open_price_with_slippage(new_direction, most_recent_quote_snapshot.last_trade_price, slippage_percentage);
      log::info!("csv: open,{eastern_now_timestamp},{:?},{hypothetcial_open_price},", new_direction);
      log::info!(
        "{eastern_now_timestamp}: opening new trade; quote_age = {quote_age}s snapshot_age = {signal_snapshot_age}s direction = {:?} signal_snapshot = {:?} quote_snapshot = {:?}",
        most_recent_direction_change,
        most_recent_signal_snapshot,
        most_recent_quote_snapshot,
      );
      // set state
      last_trade_direction = most_recent_direction_change_start_snapshot.direction;
      last_trade_open_quote.replace(most_recent_quote_snapshot.clone());
      is_trade_open = true;
    }
    // check for end of day
    if pointer == regular_market_end {
      // close any open trades
      if is_trade_open == true {
        let hypothetcial_open_price = math::calculate_open_price_with_slippage(last_trade_direction, last_trade_open_quote.as_ref().unwrap().last_trade_price, slippage_percentage);
        let hypothetical_exit_price = math::calculate_close_price_with_slippage(last_trade_direction, most_recent_quote_snapshot.last_trade_price, slippage_percentage);
        let open_profit_loss_percentage = math::calculate_profit_loss_percentage(last_trade_direction, hypothetcial_open_price, hypothetical_exit_price);
        log::info!("csv: close,{eastern_now_timestamp},{:?},{hypothetical_exit_price},EndOfDay", last_trade_direction);
        log::info!(
          "{eastern_now_timestamp}: closing trade; end of day; open_profit_loss_percentage = {open_profit_loss_percentage} quote_age = {quote_age}s current_quote = {:?} last_trade_open_quote = {:?}",
          most_recent_quote_snapshot,
          last_trade_open_quote
        );
        // mark trade closed
        total_profit_loss_percentage += open_profit_loss_percentage;
        num_trades += 1;
        is_trade_open = false;
        last_trade_open_quote = None;
      }
    }
    // increment pointer
    pointer += chrono::Duration::seconds(pointer_increment_seconds);
  }
  return (num_trades, total_profit_loss_percentage);
}

fn build_caches(connection: &Database, symbol: &str, resolution: &str, date: &str) -> (HashMap::<i64, Vec<Candle>>, HashMap::<i64, QuoteSnapshot>) {
  log::info!("building caches");
  let mut candles_timestamp_cache_map = HashMap::<i64, Vec<Candle>>::new();
  let mut quote_snapshots_timestamp_cache_map = HashMap::<i64, QuoteSnapshot>::new();
  // times
  let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
  let regular_market_start_timestamp = regular_market_start.timestamp();
  // loop over entire day second by second
  let mut pointer = regular_market_start;
  while pointer <= regular_market_end {
    // get timestamps
    let eastern_now = &pointer;
    let eastern_now_timestamp = eastern_now.timestamp();
    // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
    // let (current_candle_start, current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, eastern_now);
    // let candle_lookup_max_timestamp = current_candle_start_timestamp - 1;
    let candle_lookup_max_timestamp = eastern_now_timestamp;
    // get candles
    let candles = get_candles_from_database(&connection, symbol, resolution, regular_market_start_timestamp, candle_lookup_max_timestamp);
    candles_timestamp_cache_map.insert(eastern_now_timestamp, candles);
    // get quote snapshots
    let quote_snapshots = get_quote_snapshots_from_database(&connection, symbol, regular_market_start_timestamp, eastern_now_timestamp);
    if quote_snapshots.len() > 0 {
      let most_recent_quote_snapshot = &quote_snapshots[0];
      quote_snapshots_timestamp_cache_map.insert(eastern_now_timestamp, most_recent_quote_snapshot.clone());
    }
    // increment
    pointer += chrono::Duration::seconds(1);
  }
  return (candles_timestamp_cache_map, quote_snapshots_timestamp_cache_map);
}

fn main() {
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let symbol = "SPY";
    let resolution = "1";
    // open database
    let connection = Database::new("./database.db");
    // init database tables
    connection.migrate("./schema/");
    // time
    let date = "2023-01-31 00:00:00";
    // cache data from database
    /*let (candles_timestamp_cache_map, quote_snapshots_timestamp_cache_map) = build_caches(&connection, symbol, resolution, date);
    log::info!("writing caches to files");
    file::write_json_to_file("./cache/candles-2023-01-31.json", &candles_timestamp_cache_map).await;
    file::write_json_to_file("./cache/quotes-2023-01-31.json", &quote_snapshots_timestamp_cache_map).await;
    log::info!("wrote caches to files");*/
    // load caches from files
    log::info!("loading cache from files");
    let candles_timestamp_cache_map = file::read_json_from_file("./cache/candles-2023-01-31.json").await;
    let quote_snapshots_timestamp_cache_map = file::read_json_from_file("./cache/quotes-2023-01-31.json").await;
    log::info!("loaded cache from files");
    // build combinations
    let combinations = combinations::build_combinations();
    log::info!("num_combinations = {}", combinations.len());
    // backtest combinations
    let num_tested = std::sync::atomic::AtomicUsize::new(0);
    let mut results: Vec<(BacktestCombination, usize, f64)> = combinations.into_iter().map(|combination| {
      let (num_trades, total_profit_loss_percentage) = backtest_combination(&candles_timestamp_cache_map, &quote_snapshots_timestamp_cache_map, date, &combination);
      let num_tested = num_tested.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
      if num_tested % 100 == 0 {
        log::info!("num_tested = {}", num_tested);
      }
      return (combination, num_trades, total_profit_loss_percentage);
    })
    .collect();
    // sort by total_profit_loss_percentage descending
    results.sort_by(|a, b| {
      return b.2.partial_cmp(&a.2).unwrap();
    });
    let best_result = &results[0];
    log::info!("best_result = {:?}", best_result);
  });
}
