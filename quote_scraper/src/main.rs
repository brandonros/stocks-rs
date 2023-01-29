use chrono::{DateTime, Datelike, TimeZone, Utc, Weekday};
use chrono_tz::{Tz, US::Eastern};
use common::database;

struct QuoteSnapshot(robinhood::Quote);

impl database::ToQuery for QuoteSnapshot {
  fn insert(&self) -> (&str, Vec<(&str, &dyn rusqlite::ToSql)>) {
    let query = "
      INSERT OR REPLACE INTO quote_snapshots (
          symbol,
          scraped_at,
          ask_price,
          bid_price,
          last_trade_price
      ) VALUES (
          :symbol,
          strftime('%s', 'now'),
          :ask_price,
          :bid_price,
          :last_trade_price
      )
    ";
    let params = rusqlite::named_params! {
        ":symbol": self.0.symbol,
        ":ask_price": self.0.ask_price,
        ":bid_price": self.0.bid_price,
        ":last_trade_price": self.0.last_trade_price
    };
    return (query, params.to_vec());
  }
}

async fn align_to_top_of_second() {
  let now = Utc::now();
  let difference = 1000 - (now.timestamp_millis() % 1000);
  tokio::time::sleep(tokio::time::Duration::from_millis(difference as u64)).await;
}

fn get_regular_market_session_start_and_end(eastern_now: &DateTime<Tz>) -> (DateTime<Tz>, DateTime<Tz>) {
  let year = eastern_now.year();
  let month = eastern_now.month();
  let day = eastern_now.day();
  let regular_market_start = Eastern.with_ymd_and_hms(year, month, day, 9, 30, 0).unwrap(); // 9:30:00am
  let regular_market_end = Eastern.with_ymd_and_hms(year, month, day, 15, 59, 59).unwrap(); // 3:59:59pm
  return (regular_market_start, regular_market_end);
}

fn main() {
  // logger
  simple_logger::SimpleLogger::new().env().init().unwrap();
  // runtime
  let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
  // run
  rt.block_on(async {
    // config
    let symbol = "SPY";
    // open database
    let database = database::Database::new("./database.db");
    // init database tables
    database.migrate("./schema/");
    // get robinhood access token
    let robinhood = robinhood::Robinhood::new();
    let result = robinhood.get_logged_out_access_token().await;
    if result.is_err() {
      panic!("failed to get logged out access token: {:?}", result);
    }
    let access_token = result.unwrap();
    loop {
      // check time
      let now = Utc::now();
      let eastern_now = now.with_timezone(&Eastern);
      let (regular_market_start, regular_market_end) = get_regular_market_session_start_and_end(&eastern_now);
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
      // get quote from robinhood
      let result = robinhood.get_quote(&access_token, &symbol).await;
      if result.is_err() {
        log::error!("failed to get quote: {:?}", result);
        align_to_top_of_second().await;
        continue;
      }
      let quote = result.unwrap();
      // log
      log::info!("{:?}", quote);
      // insert quote into database
      let quote_snapshot = QuoteSnapshot(quote);
      let result = database.insert(&quote_snapshot);
      if result.is_err() {
        log::error!("failed to insert into database: {:?}", result);
        align_to_top_of_second().await;
        continue;
      }
      // sleep
      align_to_top_of_second().await;
    }
  });
}
