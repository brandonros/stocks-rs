use chrono::{Datelike, Utc, Weekday};
use chrono_tz::US::Eastern;
use common::database::Database;
use common::structs::Candle;
use common::utilities;
use common::{database, structs::QuoteSnapshot};
use strategies::supertrend::{SupertrendStrategy, SupertrendStrategyIndicatorSettings};

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
    let indicator_settings = SupertrendStrategyIndicatorSettings {
      supertrend_periods: 6,
      supertrend_multiplier: 0.25,
    };
    // open database
    let connection = database::Database::new("./database.db");
    // init database tables
    connection.migrate("./schema/");
    loop {
      // check time
      let now = Utc::now();
      let eastern_now = now.with_timezone(&Eastern);
      let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end(&eastern_now);
      // before market start
      if now < regular_market_start {
        log::warn!("now < regular_market_start");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // after market end
      if now > regular_market_end {
        log::warn!("now >= regular_market_end");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // weekend
      let weekday = eastern_now.weekday();
      let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
      if is_weekend == true {
        log::warn!("is_weekend == true");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // holiday
      let is_holiday = false; // TODO
      if is_holiday == true {
        log::warn!("is_holiday == true");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // get candles from database
      let eastern_now_timestamp = eastern_now.timestamp();
      let regular_market_start_timestamp = regular_market_start.timestamp();
      let (current_candle_start, current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, &eastern_now);
      // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle
      //let candle_lookup_max_timestamp = eastern_now_timestamp;
      let candle_lookup_max_timestamp = current_candle_start.timestamp() - 1;
      let candles = get_candles_from_database(&connection, symbol, resolution, regular_market_start_timestamp, candle_lookup_max_timestamp);
      // get most recent signal signal from candles
      let strategy = SupertrendStrategy::new();
      let signal_snapshots = strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles);
      if signal_snapshots.is_empty() {
        log::warn!("signal_snapshots.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      let most_recent_signal_snapshot = &signal_snapshots[signal_snapshots.len() - 1];
      // get current quote
      let quote_snapshots = get_quote_snapshots_from_database(&connection, symbol, regular_market_start_timestamp, eastern_now_timestamp);
      if quote_snapshots.is_empty() {
        log::warn!("quote_snapshots.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      let most_recent_quote_snapshot = &quote_snapshots[0];
      // check quote age
      let quote_age = eastern_now_timestamp - most_recent_quote_snapshot.scraped_at;
      // TODO: handle if quote_snapshot is too old/unrealistic from something like a quote_scraper process crash
      if quote_age > 1 {
        log::warn!("quote_snapshot is old!");
      }
      // check snapshot age?
      let signal_snapshot_age = eastern_now_timestamp - most_recent_signal_snapshot.candle.timestamp;
      // log
      log::info!(
        "now = {} current_candle = {}-{} quote_age = {}s snapshot_age = {}s signal_snapshot = {:?} quote_snapshot = {:?}",
        eastern_now_timestamp,
        current_candle_start.timestamp(),
        current_candle_end.timestamp(),
        quote_age,
        signal_snapshot_age,
        most_recent_signal_snapshot,
        most_recent_quote_snapshot
      );
      // TODO: insert into database?
      // TODO: paper trade based off this data?
      // sleep
      utilities::aligned_sleep(1000).await;
    }
  });
}
