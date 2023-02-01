use chrono::{DateTime, Datelike, Utc, Weekday};
use chrono_tz::{Tz, US::Eastern};
use common::{
  database::{self, ToQuery},
  structs::*,
  utilities,
};

fn main() {
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let provider_name = "tradingview";
    let symbol = "SPY";
    let resolution = "1";
    // open database
    let mut database = database::Database::new("./database.db");
    // init database tables
    database.migrate("./schema/");
    // loop
    loop {
      // check time
      let now = Utc::now();
      let eastern_now = now.with_timezone(&Eastern);
      let (regular_market_start, regular_market_end) = common::market_session::get_regular_market_session_start_and_end(&eastern_now);
      // before market start
      if now < regular_market_start {
        log::warn!("now < regular_market_start");
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // after market end
      if now > regular_market_end {
        log::warn!("now >= regular_market_end");
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // weekend
      let weekday = eastern_now.weekday();
      let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
      if is_weekend == true {
        log::warn!("is_weekend == true");
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // holiday
      let is_holiday = false; // TODO
      if is_holiday == true {
        log::warn!("is_holiday == true");
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // get candles
      let result = providers::get_candles_by_provider_name(provider_name, symbol, resolution, regular_market_start, regular_market_end).await;
      if result.is_err() {
        log::error!("failed to get candles: {:?}", result);
        utilities::aligned_sleep(5000).await;
        continue;
      }
      let candles = result.unwrap();
      if candles.is_empty() {
        log::warn!("no candles");
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // check age
      let oldest_candle = &candles[0];
      let most_recent_candle = &candles[candles.len() - 1];
      let (current_candle_start, _current_candle_end) = common::market_session::get_current_candle_start_and_stop(resolution, &eastern_now);
      let current_candle_start_timestamp = current_candle_start.timestamp();
      let age = current_candle_start_timestamp - most_recent_candle.timestamp;
      if most_recent_candle.timestamp != current_candle_start_timestamp {
        log::warn!(
          "did not scrape most recent candle? {} != {} (differnece {}s)",
          most_recent_candle.timestamp,
          current_candle_start_timestamp,
          age
        );
      }
      // log
      log::info!("scraped {} candles; most_recent_candle = {:?} oldest_candle = {:?}", candles.len(), most_recent_candle, oldest_candle);
      // insert most recent candle into database
      // TODO: only update most recent candle or go back and update all?
      let result = database.batch_insert(&candles);
      if result.is_err() {
        log::error!("failed to insert rows into database {:?}", result);
        utilities::aligned_sleep(5000).await;
        continue;
      }
      // TODO: store more than just the most recent candle?
      // sleep
      utilities::aligned_sleep(5000).await;
    }
  });
}
