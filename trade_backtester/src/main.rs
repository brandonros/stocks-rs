use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use common::database;
use common::database::Database;
use common::dates;
use common::market_session;
use common::math;
use common::structs::*;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
struct BacktestContext {
  pub slippage_percentage: f64,
  pub stop_loss_percentage: f64,
  pub profit_limit_percentage: f64
}

#[derive(Serialize, Debug)]
enum Outcome {
  StopLoss,
  ProfitLimit,
  DirectionChange,
}

#[derive(Serialize, Debug)]
struct TradeResult {
  pub direction: Direction,
  pub start_timestamp: i64,
  pub exit_timestamp: i64,
  pub outcome: Outcome,
  pub open_price: f64,
  pub exit_price: f64,
  pub profit_loss: f64,
  pub profit_loss_percentage: f64,
}

fn get_candle_snapshots_from_database(
  connection: &Database,
  symbol: &str,
  resolution: &str,
  regular_market_start_timestamp: i64,
  regular_market_end_timestamp: i64,
) -> Vec<CandleSnapshot> {
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
    where timestamp >= {regular_market_start_timestamp} and timestamp <= {regular_market_end_timestamp}
    and symbol = '{symbol}'
    and resolution = '{resolution}'
    ORDER BY timestamp ASC
    "
  );
  // TODO: filter out current partial candle and only look at 100% closed candles?
  // TODO: how to check if candle_scraper process crashed and data is stale/partial?
  return connection.get_rows_from_database::<CandleSnapshot>(&query);
}

fn calculate_trade_result(
  backtest_context: &BacktestContext,
  trade_candles: &Vec<Arc<Candle>>,
  trade_direction: &Direction,
  start_timestamp: i64,
  open_price: f64
) -> TradeResult {
  // backtest context variables
  let slippage_percentage = backtest_context.slippage_percentage;
  let stop_loss_percentage = backtest_context.stop_loss_percentage;
  let profit_limit_percentage = backtest_context.profit_limit_percentage;
  // stop loss/profit limit prices
  let stop_loss_price = math::calculate_stop_loss_price(trade_direction, open_price, stop_loss_percentage);
  let profit_limit_price = math::calculate_profit_limit_price(trade_direction, open_price, profit_limit_percentage);
  for i in 0..trade_candles.len() {
    let trade_candle = &trade_candles[i];
    // worst case scenario first based on direction for stop loss
    let exit_price = if *trade_direction == Direction::Long {
      trade_candle.low
    } else {
      trade_candle.high
    };
    let open_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
    let stop_loss_hit = open_profit_loss_percentage <= stop_loss_percentage;
    if stop_loss_hit {    
      // force exit price to be capped to exactly a perfect fill stop_loss_price at worse
      let exit_price = stop_loss_price;
      let profit_loss = math::calculate_profit_loss(trade_direction, open_price, exit_price);
      let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
      return TradeResult {
        open_price,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        outcome: Outcome::StopLoss,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
    // best case scenario next based on direction for profit limit
    let exit_price = if *trade_direction == Direction::Long {
      trade_candle.high
    } else {
      trade_candle.low
    };
    let open_profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
    let profit_limit_hit = open_profit_loss_percentage >= profit_limit_percentage;
    if profit_limit_hit {
      // force exit price to be capped to exactly a perfect fill profit_limit_price at best
      let exit_price = profit_limit_price;
      let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, exit_price);
      let profit_loss_percentage = math::calculate_profit_loss_percentage(trade_direction, open_price, exit_price);
      return TradeResult {
        open_price,
        direction: trade_direction.clone(),
        start_timestamp,
        exit_timestamp: trade_candle.timestamp,
        outcome: Outcome::ProfitLimit,
        exit_price,
        profit_loss,
        profit_loss_percentage,
      };
    }
  }
  // exit on last candle close (TODO: this is probably unrealistic and it'd be the next candle open that happens 1 second later (on registered direction change))
  let trade_end_candle = &trade_candles[trade_candles.len() - 1];
  let exit_price = trade_end_candle.close;
  let exit_price = math::calculate_close_price_with_slippage(&trade_direction, exit_price, slippage_percentage);
  let profit_loss = math::calculate_profit_loss(&trade_direction, open_price, exit_price);
  let profit_loss_percentage = math::calculate_profit_loss_percentage(&trade_direction, open_price, exit_price);
  // direction change within range of stop loss and profit limit
  return TradeResult {
    open_price,
    direction: trade_direction.clone(),
    start_timestamp,
    exit_timestamp: trade_end_candle.timestamp,
    outcome: Outcome::DirectionChange,
    exit_price,
    profit_loss,
    profit_loss_percentage,
  };
}

fn debug_trade_result(trade_result: &TradeResult) {
  let trade_result_type = if trade_result.profit_loss > 0.0 {
    String::from("win")
  } else {
    String::from("loss")
  };
  let mut row = vec![];
  row.push(format!("{}", dates::format_timestamp(trade_result.start_timestamp)));
  row.push(format!("{:?}", trade_result.direction));
  row.push(format!("${:.2}", trade_result.open_price));
  row.push(format!("{:?}", trade_result.outcome));
  row.push(format!("{}", dates::format_timestamp(trade_result.exit_timestamp)));
  row.push(format!("${:.2}", trade_result.exit_price));
  row.push(format!("${:.2}", trade_result.profit_loss));
  row.push(format!("{:.4}", trade_result.profit_loss_percentage));
  row.push(trade_result_type);
  log::info!("{}", row.join(","));
}

fn main() {
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // config
  let args: Vec<String> = std::env::args().collect();
  let provider_name = args.get(1).unwrap();
  let strategy_name = args.get(2).unwrap();
  let symbol = args.get(3).unwrap();
  let resolution = args.get(4).unwrap();
  let dates_start = format!("{} 00:00:00", args.get(5).unwrap());
  let dates_end = format!("{} 00:00:00", args.get(6).unwrap());
  let dates = common::dates::build_list_of_dates(&dates_start, &dates_end);
  // open database + init database tables
  let database_filename = format!("./database-{}.db", provider_name);
  let connection = database::Database::new(&database_filename);
  connection.migrate("./schema/");
  // build candles cache map
  let mut candles_date_map = HashMap::new();
  let mut candles_timestamp_map = HashMap::new();
  for date in &dates {
    let (regular_market_start, regular_market_end) = market_session::get_regular_market_session_start_and_end_from_string(date);
    let regular_market_start_timestamp = regular_market_start.timestamp();
    let regular_market_end_timestamp = regular_market_end.timestamp();
    // get candles from database
    let candle_snapshots = get_candle_snapshots_from_database(&connection, symbol, resolution, regular_market_start_timestamp, regular_market_end_timestamp);
    let candles: Vec<Arc<Candle>> = candle_snapshots.iter().map(|candle_snapshot| {
      return Arc::new(Candle {
        timestamp: candle_snapshot.timestamp,
        open: candle_snapshot.open,
        high: candle_snapshot.high,
        low: candle_snapshot.low,
        close: candle_snapshot.close,
        volume: candle_snapshot.volume as i64,
      });
    }).collect();
    let mut date_candles: Vec<Arc<Candle>> = vec![];
    for candle in candles {
      candles_timestamp_map.insert(candle.timestamp, candle.clone());
      date_candles.push(candle.clone());
    }
    candles_date_map.insert(date.clone(), date_candles);
  }
  // read list of trades
  let stringified_trades = std::fs::read_to_string(format!("/tmp/{}-trades.json", strategy_name)).unwrap();
  let dates_trades_map: HashMap<String, Vec<Trade>> = serde_json::from_str(&stringified_trades).unwrap();
  // backtest trades
  let backtest_context = BacktestContext {
    slippage_percentage: 0.000125,
    profit_limit_percentage: 0.004,
    stop_loss_percentage: -0.002
  };
  let starting_balance = 1000.00;
  let mut balance = starting_balance;
  let mut dates_trades_results_map = HashMap::new();
  for date in &dates {
    let date_candles = candles_date_map.get(date).unwrap();
    let date_trades = dates_trades_map.get(date).unwrap();
    let date_trade_results: Vec<TradeResult> = date_trades.iter().map(|trade| {
      let trade_candles: Vec<Arc<Candle>> = date_candles
        .iter()
        .filter(|candle| {
          return candle.timestamp >= trade.start_timestamp && candle.timestamp <= trade.end_timestamp;
        })
        .cloned()
        .collect();
        let open_candle = &trade_candles[0];
        let slippage_percentage = backtest_context.slippage_percentage;
        let open_price = math::calculate_open_price_with_slippage(&trade.direction, open_candle.open, slippage_percentage);
        return calculate_trade_result(
          &backtest_context,
          &trade_candles,
          &trade.direction,
          open_candle.timestamp,
          open_price
        );
    })
    .collect();
    for date_trade_result in &date_trade_results {
      balance *= 1.0 + date_trade_result.profit_loss_percentage;
    }
    dates_trades_results_map.insert(date.clone(), date_trade_results);
  }
  // write to file
  let stringified_value = serde_json::to_string_pretty(&dates_trades_results_map).unwrap();
  let mut file = std::fs::File::create(format!("/tmp/{}-trade-results.json", strategy_name)).unwrap();
  file.write_all(stringified_value.as_bytes()).unwrap();
  // print result
  let compounded_profit_loss_percentage = math::calculate_percentage_increase(starting_balance, balance);
  log::info!("compounded_profit_loss_percentage = {:.2}", compounded_profit_loss_percentage);
}
