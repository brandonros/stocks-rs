use anyhow::Result;
use chrono::{DateTime, TimeZone};
use chrono_tz::Tz;
use common::structs::*;
use serde::Deserialize;

/*
#!/bin/bash
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month1&apikey=KLRE95OO3G0LUEZU" -o year1month1.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month2&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month2.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month3&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month3.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month4&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month4.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month5&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month5.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month6&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month6.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month7&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month7.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month8&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month8.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month9&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month9.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month10&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month10.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month11&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month11.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year1month12&apikey=$ALPHA_VANTAGE_API_KEY" -o year1month12.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month1&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month1.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month2&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month2.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month3&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month3.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month4&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month4.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month5&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month5.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month6&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month6.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month7&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month7.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month8&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month8.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month9&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month9.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month10&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month10.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month11&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month11.csv
sleep 15
curl "https://www.alphavantage.co/query?function=TIME_SERIES_INTRADAY_EXTENDED&symbol=SPY&interval=1min&slice=year2month12&apikey=$ALPHA_VANTAGE_API_KEY" -o year2month12.csv
sleep 15
*/

#[derive(Deserialize)]
struct AlphaVantageCsvRow {
  time: String,
  open: f64,
  high: f64,
  low: f64,
  close: f64,
  volume: usize
}

pub struct AlphaVantage {
  
}

impl AlphaVantage {
  pub fn new() -> AlphaVantage {
    return AlphaVantage {
      
    };
  }

  pub async fn get_candles(&self, symbol: &str, resolution: &str, from: DateTime<Tz>, to: DateTime<Tz>) -> Result<Vec<Candle>> {
    // TODO: actually call API? it's 1 request every 15 seconds for free, just load from CSV into SQLIte here?
    assert!(symbol == "SPY");
    assert!(resolution == "1");
    let filenames = vec![
      "./data/year1month1.csv",
      "./data/year1month2.csv",
      "./data/year1month3.csv",
      "./data/year1month4.csv",
      "./data/year1month5.csv",
      "./data/year1month6.csv",
      "./data/year1month7.csv",
      "./data/year1month8.csv",
      "./data/year1month9.csv",
      "./data/year1month10.csv",
      "./data/year1month11.csv",
      "./data/year1month12.csv",
      "./data/year2month1.csv",
      "./data/year2month2.csv",
      "./data/year2month3.csv",
      "./data/year2month4.csv",
      "./data/year2month5.csv",
      "./data/year2month6.csv",
      "./data/year2month7.csv",
      "./data/year2month8.csv",
      "./data/year2month9.csv",
      "./data/year2month10.csv",
      "./data/year2month11.csv",
      "./data/year2month12.csv",
    ];
    let mut candles: Vec<Candle> = vec![];
    for filename in filenames.iter() {
      let file_path = std::path::Path::new(filename);
      let file = std::fs::File::open(file_path)?;
      let mut csv_reader = csv::ReaderBuilder::new()
          .has_headers(true)
          .from_reader(file);
      let rows: Vec<AlphaVantageCsvRow> = csv_reader.deserialize::<AlphaVantageCsvRow>().collect::<Result<_, _>>()?;
      let mut mapped_rows = rows.into_iter().map(|row| {
        let parsed_time = chrono::NaiveDateTime::parse_from_str(&row.time, "%Y-%m-%d %H:%M:%S").unwrap();
        let parsed_time = chrono_tz::US::Eastern.from_local_datetime(&parsed_time).unwrap();
        // TODO: validate timestamp is correct timezone + parsed correctly?
        return Candle {
          timestamp: parsed_time.timestamp(),
          open: row.open,
          high: row.high,
          low: row.low,
          close: row.close,
          volume: row.volume as i64
        };
      }).collect::<Vec<_>>();
      candles.append(&mut mapped_rows);
    }
    let mut candles = candles.into_iter().filter(|candle| {
      return candle.timestamp >= from.timestamp() && candle.timestamp <= to.timestamp();
    }).collect::<Vec<_>>();
    candles.sort_by(|a, b| {
      return a.timestamp.partial_cmp(&b.timestamp).unwrap();
    });
    return Ok(candles);
  }
}
