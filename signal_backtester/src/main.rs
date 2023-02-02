use chrono_tz::US::Eastern;
use common::database::Database;
use common::{structs::*, math};
use common::{database, structs::QuoteSnapshot};
use strategies::supertrend::{SupertrendStrategy, SupertrendStrategyIndicatorSettings};

fn get_candle_snapshots_from_database(connection: &Database, symbol: &str, resolution: &str, eastern_now_timestamp: i64, regular_market_start_timestamp: i64, candle_lookup_max_timestamp: i64) -> Vec<CandleSnapshot> {
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
    and scraped_at = (select scraped_at from candles where scraped_at >= {regular_market_start_timestamp} and scraped_at <= {eastern_now_timestamp} order by scraped_at desc limit 1) 
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
  "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

fn get_quote_snapshots_from_database(connection: &Database, symbol: &str, start_timestamp: i64, end_timestamp: i64) -> Vec<QuoteSnapshot> {
  let quotes_query = format!(
    "
    select scraped_at, ask_price, bid_price, last_trade_price
    from quote_snapshots
    where symbol = '{symbol}' and scraped_at >= {start_timestamp} and scraped_at <= {end_timestamp}
    order by scraped_at asc
    "
  );
  let quote_snapshots = connection.get_rows_from_database::<QuoteSnapshot>(&quotes_query);
  return quote_snapshots;
}

fn backtest_date(connection: &Database, symbol: &str, resolution: &str, warmed_up_index: usize, indicator_settings: &SupertrendStrategyIndicatorSettings, date: &str) -> Result<(), String> {
  let slippage_percentage = 0.000125;
  let profit_limit_percentage = 0.005;
  let stop_loss_percentage = -0.01;
  let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end_from_string(date);
  // get mock now from end of day TODO: no need to walk through entire day second by second/minute by minute? use end of day direction changes as "collective" answer to what happened throughout the day
  let eastern_now = regular_market_end.with_timezone(&Eastern);
  let eastern_now_timestamp = eastern_now.timestamp();
  let regular_market_start_timestamp = regular_market_start.timestamp();
  let (current_candle_start, current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, &eastern_now);
  let current_candle_index = (eastern_now_timestamp - regular_market_start_timestamp) / 60;
  // get quotes from database
  let quote_snapshots = get_quote_snapshots_from_database(connection, symbol, regular_market_start_timestamp, eastern_now_timestamp);
  // get candles from database
  // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
  //let candle_lookup_max_timestamp = eastern_now_timestamp;
  let candle_lookup_max_timestamp = current_candle_start.timestamp() - 1;
  let candle_snapshots = get_candle_snapshots_from_database(&connection, symbol, resolution, eastern_now_timestamp, regular_market_start_timestamp, candle_lookup_max_timestamp);
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
    log::warn!("{eastern_now_timestamp} ({current_candle_index}/390): signal_snapshot candle is old! most_recent_signal_snapshot_candle_age = {}", most_recent_signal_snapshot_candle_age);
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
    let start_signal_snapshot = &trade_signal_snapshots[0];
    let end_signal_snapshot = &trade_signal_snapshots[trade_signal_snapshots.len() - 1];
    log::info!("{:?} {:?}", start_signal_snapshot.direction, end_signal_snapshot.direction);
    let trade_start_timestamp = start_signal_snapshot.candle.timestamp;
    let trade_end_timestamp = end_signal_snapshot.candle.timestamp + 59; // TODO: add 59 seconds here?
    let mut quotes_portfolio_balance = 1000.00;
    let mut candles_portfolio_balance = 1000.00;

    let trade_quote_snapshots = get_quote_snapshots_from_database(connection, symbol, trade_start_timestamp, trade_end_timestamp);
    let start_quote_snapshot = &trade_quote_snapshots[0];
    let end_quote_snapshot = &trade_quote_snapshots[trade_quote_snapshots.len() - 1];
    let quote_open_price = math::calculate_open_price_with_slippage(start_signal_snapshot.direction, start_quote_snapshot.last_trade_price, slippage_percentage);
    let quote_exit_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, end_quote_snapshot.last_trade_price, slippage_percentage);
    let quote_exit_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, quote_open_price, quote_exit_price);
    let mut exit_quote_result = (BacktestOutcome::DirectionChange, end_quote_snapshot.clone(), quote_exit_price, quote_exit_profit_loss_percentage);
    for trade_quote_snapshot in &trade_quote_snapshots {
      let hypothetical_exit_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, trade_quote_snapshot.last_trade_price, slippage_percentage);
      let hypothetical_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, quote_open_price, hypothetical_exit_price);
      // always check stop loss first to be as pessimistic as possible
      if hypothetical_profit_loss_percentage <= stop_loss_percentage {
        exit_quote_result = (BacktestOutcome::StopLoss, trade_quote_snapshot.clone(), hypothetical_exit_price, stop_loss_percentage); // assumes we would do no better/no worse than exactly stop loss percentage?
        break;
      }
      if hypothetical_profit_loss_percentage >= profit_limit_percentage {
        exit_quote_result = (BacktestOutcome::ProfitLimit, trade_quote_snapshot.clone(), hypothetical_exit_price, profit_limit_percentage); // assumes we would do no better/no worse than exactly profit limit percentage?
        break;
      }
    }
    quotes_portfolio_balance *= (1.0 + exit_quote_result.3);

    let candle_open_price = math::calculate_open_price_with_slippage(start_signal_snapshot.direction, start_signal_snapshot.candle.open, slippage_percentage);
    let candle_exit_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, end_signal_snapshot.candle.close, slippage_percentage); // TODO: would probably get out on next candle open instead of last candle close? but we don't include this next candle on purpose?
    let candle_exit_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, candle_open_price, candle_exit_price);
    let mut exit_candle_result = (BacktestOutcome::DirectionChange, end_signal_snapshot.clone(), candle_exit_price, candle_exit_profit_loss_percentage);
    for trade_signal_snapshot in trade_signal_snapshots {
      let best_case_scenario_price = math::calculate_best_case_scenario_price(start_signal_snapshot.direction, &trade_signal_snapshot.candle);
      let worst_case_scenario_price = math::calculate_worst_case_scenario_price(start_signal_snapshot.direction, &trade_signal_snapshot.candle);
      let hypothetical_best_case_scenario_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, best_case_scenario_price, slippage_percentage);
      let hypothetical_worst_case_scenario_price = math::calculate_close_price_with_slippage(start_signal_snapshot.direction, worst_case_scenario_price, slippage_percentage);
      let hypothetical_best_case_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, candle_open_price, hypothetical_best_case_scenario_price);
      let hypothetical_worst_case_profit_loss_percentage = math::calculate_profit_loss_percentage(start_signal_snapshot.direction, candle_open_price, hypothetical_worst_case_scenario_price);
      // always check stop loss first to be as pessimistic as possible
      if hypothetical_worst_case_profit_loss_percentage <= stop_loss_percentage {
        exit_candle_result = (BacktestOutcome::StopLoss, trade_signal_snapshot.clone(), hypothetical_worst_case_scenario_price, stop_loss_percentage); // assumes we would do no better/no worse than exactly stop loss percentage?
        break;
      }
      if hypothetical_best_case_profit_loss_percentage >= profit_limit_percentage {
        exit_candle_result = (BacktestOutcome::ProfitLimit, trade_signal_snapshot.clone(), hypothetical_best_case_scenario_price, profit_limit_percentage); // assumes we would do no better/no worse than exactly profit limit percentage?
        break;
      }
    }
    candles_portfolio_balance *= (1.0 + exit_candle_result.3);

    results.push(serde_json::json!({
      "direction": start_signal_snapshot.direction,
      "mode": "quote",
      "start": {
        "timestamp": start_quote_snapshot.scraped_at,
        "price": start_quote_snapshot.last_trade_price
      },
      "end": {
        "timestamp": end_quote_snapshot.scraped_at,
        "price": end_quote_snapshot.last_trade_price
      },
      "exit": {
        "outcome": exit_quote_result.0,
        "timestamp": exit_quote_result.1.scraped_at,
        "price": exit_quote_result.2,
        "profit_loss_percentage": exit_quote_result.3,
      },
      "balance": quotes_portfolio_balance
    }));
    results.push(serde_json::json!({
      "direction": start_signal_snapshot.direction,
      "mode": "candle",
      "start": {
        "timestamp": start_signal_snapshot.candle.timestamp,
        "price": start_signal_snapshot.candle.open
      },
      "end": {
        "timestamp": end_signal_snapshot.candle.timestamp,
        "price": end_signal_snapshot.candle.open
      },
      "exit": {
        "outcome": exit_candle_result.0,
        "timestamp": exit_candle_result.1.candle.timestamp,
        "price": exit_candle_result.2,
        "profit_loss_percentage": exit_candle_result.3,
      },
      "balance": candles_portfolio_balance
    }));
  }
  log::info!("{}", serde_json::to_string(&results).unwrap());
  return Ok(());
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
    let warmed_up_index = 10; // TODO: 10 or 9 or 0 or something different?
    let indicator_settings = SupertrendStrategyIndicatorSettings {
      supertrend_periods: 10,
      supertrend_multiplier: 3.00,
    };
    // open database
    let connection = database::Database::new("./database.db");
    // init database tables
    connection.migrate("./schema/");
    // backtest
    let date = "2023-02-01 00:00:00";    
    let result = backtest_date(&connection, symbol, resolution, warmed_up_index, &indicator_settings, date);
  });
}
