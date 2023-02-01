use chrono::{Datelike, Utc, Weekday};
use chrono_tz::US::Eastern;
use common::database;
use common::utilities;
use providers::robinhood;

struct QuoteSnapshot(robinhood::structs::Quote);

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

fn main() {
  // logger
  simple_logger::init_with_level(log::Level::Info).unwrap();
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
    // TODO: this expires every hour and causes the process to exit, we could probably be smarter because exiting and restart causes us to get behind quote snapshot age wise 2-5 seconds
    let result = robinhood.get_logged_out_access_token().await;
    if result.is_err() {
      panic!("failed to get logged out access token: {:?}", result);
    }
    let access_token = result.unwrap();
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
      // get quote from robinhood
      // TODO: handle failed to get quote Err("unknown error: invalid response status: 401") better due to token expiring
      let result = robinhood.get_quote(&access_token, symbol).await;
      if result.is_err() {
        panic!("failed to get quote {:?}", result);
      }
      let quote = result.unwrap();
      // log
      log::info!("{:?}", quote);
      // insert quote into database
      let quote_snapshot = QuoteSnapshot(quote);
      let result = database.insert(&quote_snapshot);
      if result.is_err() {
        log::error!("failed to insert into database: {:?}", result);
        utilities::aligned_sleep(1000).await;
        continue;
      }
      // sleep
      utilities::aligned_sleep(1000).await;
    }
  });
}
