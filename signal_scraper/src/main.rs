use chrono::{Datelike, Utc, Weekday};
use chrono_tz::US::Eastern;
use common::{database, structs::QuoteSnapshot};
use common::structs::Candle;
use common::utilities;
use strategies::supertrend::{SupertrendStrategy, SupertrendStrategyIndicatorSettings};

fn main() {
  // logger
  simple_logger::SimpleLogger::new().env().init().unwrap();
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
    let database = database::Database::new("./database.db");
    // init database tables
    database.migrate("./schema/");
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
      let candles_query = format!("
        with most_recent_candle_snapshots as (
          select max(scraped_at) as scraped_at, symbol, timestamp, resolution from candles
          where scraped_at <= {eastern_now_timestamp}
          group by symbol, timestamp, resolution
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
        from candles
        join most_recent_candle_snapshots on most_recent_candle_snapshots.scraped_at = candles.scraped_at and 
          most_recent_candle_snapshots.timestamp = candles.timestamp and 
          most_recent_candle_snapshots.symbol = candles.symbol and 
          most_recent_candle_snapshots.resolution = candles.resolution
          where candles.timestamp >= {regular_market_start_timestamp}
          and candles.resolution = '{resolution}'
          and candles.symbol = '{symbol}'
        ORDER BY candles.timestamp ASC
      ");
      // TODO: filter out current partial candle and only look at 100% closed candles?
      // TODO: how to check if candle_scraper process crashed and data is stale/partial?
      let candles = database.get_rows_from_database::<Candle>(&candles_query);
      // get most recent signal signal from candles
      let strategy = SupertrendStrategy::new();
      let signal_snapshots = strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles);
      if signal_snapshots.len() == 0 {
        log::warn!("signal_snapshots.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      let most_recent_signal_snapshot = &signal_snapshots[signal_snapshots.len() - 1];
      // get current quote
      let quotes_query = format!(
        "
        select symbol, scraped_at, ask_price, bid_price, last_trade_price
        from quote_snapshots
        where symbol = '{symbol}' and scraped_at <= {eastern_now_timestamp}
        order by scraped_at desc
        limit 1
        "
      );
      let quote_snapshots = database.get_rows_from_database::<QuoteSnapshot>(&quotes_query);
      if quote_snapshots.len() == 0 {
        log::warn!("quote_snapshots.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      let most_recent_quote_snapshot = &quote_snapshots[0];
      // TODO: handle if quote_snapshot is too old/unrealistic from something like a quote_scraper process crash
      // log
      log::info!("{}: {:?} {:?}", eastern_now_timestamp, most_recent_signal_snapshot, most_recent_quote_snapshot);
      // TODO: insert into database?
      // sleep
      utilities::aligned_sleep(1000).await;
    }
  });
}
