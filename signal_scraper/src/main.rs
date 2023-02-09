use chrono::{Datelike, Utc, Weekday};
use chrono_tz::US::Eastern;
use common::cache;
use common::candles;
use common::database::Database;
use common::structs::*;
use common::utilities;
use common::{database, structs::QuoteSnapshot};
use strategies::supertrend::{SupertrendStrategy, SupertrendStrategyIndicatorSettings};

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
      warmed_up_index: 10, // TODO: 10 or 0 or something different?
      supertrend_periods: 10,
      supertrend_multiplier: 3.00,
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
      // TODO: which is better, follow current timestamp with no delay or always look to previous closed candle?
      //let candle_lookup_max_timestamp = eastern_now_timestamp;
      let candle_lookup_max_timestamp = current_candle_start.timestamp() - 1;
      let candle_snapshots = candles::get_live_candle_snapshots_from_database(
        &connection,
        symbol,
        resolution,
        eastern_now_timestamp,
        regular_market_start_timestamp,
        candle_lookup_max_timestamp,
      );
      if candle_snapshots.len() == 0 {
        log::warn!("candles.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // convert candle snapshots to candles
      let candles = candle_snapshots
        .into_iter()
        .map(|candle_snapshot| {
          return Candle {
            timestamp: candle_snapshot.timestamp,
            open: candle_snapshot.open,
            high: candle_snapshot.high,
            low: candle_snapshot.low,
            close: candle_snapshot.close,
            volume: candle_snapshot.volume,
          };
        })
        .collect();
      // get recent signal signal from candles
      let strategy = SupertrendStrategy::new();
      let signal_snapshots = strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles);
      if signal_snapshots.is_empty() {
        log::warn!("signal_snapshots.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // get direction changes
      let direction_changes = strategies::build_direction_changes_from_signal_snapshots(&signal_snapshots, indicator_settings.warmed_up_index);
      if direction_changes.is_empty() {
        log::warn!("direction_changes.len() == 0");
        utilities::aligned_sleep(1000).await;
        continue;
      }
      let enriched_direction_changes = strategies::build_enriched_direction_changes(&direction_changes, &signal_snapshots);
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
        log::warn!("quote_snapshot is old! quote_age = {}", quote_age);
      }
      // check snapshot age?
      let most_recent_signal_snapshot = &signal_snapshots[signal_snapshots.len() - 1];
      let most_recent_signal_snapshot_candle_age = eastern_now_timestamp - most_recent_signal_snapshot.candle.timestamp;
      if most_recent_signal_snapshot_candle_age > 120 {
        log::warn!(
          "signal_snapshot candle is old! most_recent_signal_snapshot_candle_age = {}",
          most_recent_signal_snapshot_candle_age
        );
      }
      // log
      let current_candle_index = (eastern_now_timestamp - regular_market_start_timestamp) / 60;
      let most_recent_enriched_direction_change = &enriched_direction_changes[enriched_direction_changes.len() - 1];
      let previous_enriched_direction_changes = &enriched_direction_changes[0..enriched_direction_changes.len() - 1];
      // log
      log::info!(
        "{}",
        serde_json::to_string(&serde_json::json!({
          "now": eastern_now_timestamp,
          "current_candle": {
            "index": current_candle_index,
            "start": current_candle_start.timestamp(),
            "end": current_candle_end.timestamp()
          },
          "quote": {
            "age": quote_age,
            "snapshot": most_recent_quote_snapshot
          },
          "settings": indicator_settings,
          "signal": {
            "candle_age": most_recent_signal_snapshot_candle_age,
            "snapshot": most_recent_signal_snapshot
          },
          "direction_changes": {
            "current": most_recent_enriched_direction_change,
            "previous": previous_enriched_direction_changes
          }
        }))
        .unwrap()
      );
      // TODO: insert into database?
      // TODO: paper trade based off this data?*/
      // sleep
      utilities::aligned_sleep(1000).await;
    }
  });
}
