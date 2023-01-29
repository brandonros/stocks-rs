use rusqlite::{named_params, Connection};

use crate::structs::Candle;

pub fn get_database_connection(provider_name: &str) -> Connection {
  return rusqlite::Connection::open(format!("./backtester-{provider_name}.db")).unwrap();
}

pub fn init_tables(connection: &Connection) {
  connection
    .execute(
      "
    CREATE TABLE IF NOT EXISTS candles (
      symbol TEXT,
      resolution TEXT,
      timestamp INTEGER,
      open REAL,
      high REAL,
      low REAL,
      close REAL,
      volume INTEGER,
      PRIMARY KEY (symbol, resolution, timestamp)
    )
  ",
      (),
    )
    .unwrap();
}

pub fn insert_candles_to_database(connection: &Connection, symbol: &str, resolution: &str, candles: &Vec<Candle>) {
  log::info!("inserting {} candles", candles.len());
  for candle in candles {
    // TODO: use a query builder
    connection
      .execute(
        "
        INSERT OR REPLACE INTO candles (
            symbol,
            resolution,
            timestamp,
            open,
            high,
            low,
            close,
            volume
        ) VALUES (
            :symbol,
            :resolution,
            :timestamp,
            :open,
            :high,
            :low,
            :close,
            :volume
        )
        ",
        named_params! {
            ":symbol": symbol,
            ":resolution": resolution,
            ":timestamp": candle.timestamp,
            ":open": candle.open,
            ":high": candle.high,
            ":low": candle.low,
            ":close": candle.close,
            ":volume": candle.volume
        },
      )
      .unwrap();
  }
}

pub fn get_rows_from_database<T>(connection: &Connection, query: &str) -> Vec<T>
where
  T: for<'de> serde::Deserialize<'de>,
{
  let mut statement = connection.prepare(query).unwrap();
  let column_count = statement.column_count();
  let mut rows = statement.query(()).unwrap();
  let mut results = Vec::new();
  while let Some(row) = rows.next().unwrap() {
    let mut row_map = serde_json::map::Map::new();
    for column_index in 0..column_count {
      let column_name = row.as_ref().column_name(column_index).unwrap().to_string();
      let sqlite_value_ref = row.get_ref_unwrap(column_index);
      match sqlite_value_ref {
        rusqlite::types::ValueRef::Null => todo!(),
        rusqlite::types::ValueRef::Blob(_) => todo!(),
        rusqlite::types::ValueRef::Integer(sqlite_value) => {
          let json_value = serde_json::Value::from(sqlite_value);
          row_map.insert(column_name, json_value);
        }
        rusqlite::types::ValueRef::Real(sqlite_value) => {
          let json_value = serde_json::Value::from(sqlite_value);
          row_map.insert(column_name, json_value);
        }
        rusqlite::types::ValueRef::Text(sqlite_value) => {
          let str_value = std::str::from_utf8(sqlite_value).unwrap();
          let json_value = serde_json::Value::from(str_value);
          row_map.insert(column_name, json_value);
        }
      }
    }
    let converted_row_map = serde_json::from_value::<T>(serde_json::Value::Object(row_map)).unwrap();
    results.push(converted_row_map);
  }
  return results;
}
