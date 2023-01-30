use chrono::{DateTime, Datelike, Utc, Weekday};
use chrono_tz::{Tz, US::Eastern};
use common::{database, structs::*};

async fn align_to_top_of_second() {
  let now = Utc::now();
  let difference = 1000 - (now.timestamp_millis() % 1000);
  tokio::time::sleep(tokio::time::Duration::from_millis(difference as u64)).await;
}

// TODO: convert this to a trait?
async fn get_candles_by_provider_name(
  provider_name: &str,
  symbol: &str,
  resolution: &str,
  from: DateTime<Tz>,
  to: DateTime<Tz>,
) -> Result<Vec<Candle>, String> {
  match provider_name {
    "yahoo_finance" => {
      let provider = providers::yahoo_finance::YahooFinance::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    "finnhub" => {
      let provider = providers::finnhub::Finnhub::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    "polygon" => {
      let provider = providers::polygon::Polygon::new();
      let result = provider.get_candles(symbol, resolution, from, to).await;
      if result.is_err() {
        return Err(format!("{:?}", result));
      }
      return Ok(result.unwrap());
    }
    _ => unimplemented!(),
  }
}

fn main() {
  // logger
  simple_logger::SimpleLogger::new().env().init().unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let provider_name = "yahoo_finance";
    let symbol = "SPY";
    let resolution = "1";
    // open database
    let database = database::Database::new("./database.db");
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
        align_to_top_of_second().await;
        continue;
      }
      // after market end
      if now > regular_market_end {
        log::warn!("now >= regular_market_end");
        align_to_top_of_second().await;
        continue;
      }
      // weekend
      let weekday = eastern_now.weekday();
      let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
      if is_weekend == true {
        log::warn!("is_weekend == true");
        align_to_top_of_second().await;
        continue;
      }
      // holiday
      let is_holiday = false; // TODO
      if is_holiday == true {
        log::warn!("is_holiday == true");
        align_to_top_of_second().await;
        continue;
      }
      // get candle
      // TODO: support scraping historical candles?
      let result = get_candles_by_provider_name(provider_name, symbol, resolution, regular_market_start, regular_market_end).await;
      if result.is_err() {
        log::error!("failed to get candles: {:?}", result);
        align_to_top_of_second().await;
        continue;
      }
      let candles = result.unwrap();
      if candles.len() == 0 {
        log::warn!("no candles");
        align_to_top_of_second().await;
        continue;
      }
      let most_recent_candle = &candles[candles.len() - 1];
      // log
      log::info!("{:?}", most_recent_candle);
      // insert most recent candle into database
      let result = database.insert(most_recent_candle);
      if result.is_err() {
        log::error!("failed to insert into database: {:?}", result);
        align_to_top_of_second().await;
        continue;
      }
      // TODO: store more than just the most recent candle?
      // sleep
      align_to_top_of_second().await;
    }
  });
}
