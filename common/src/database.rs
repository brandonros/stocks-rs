pub trait ToQuery {
  fn insert(&self) -> (&str, Vec<(&str, &dyn rusqlite::ToSql)>);
}

pub struct Database {
  connection: rusqlite::Connection,
}

impl Database {
  pub fn new(filename: &str) -> Database {
    return Database {
      connection: rusqlite::Connection::open(filename).unwrap(),
    };
  }

  pub fn execute_query<P: rusqlite::Params>(&self, sql: &str, params: P) -> Result<usize, rusqlite::Error> {
    return self.connection.execute(sql, params);
  }

  pub fn get_rows_from_database<T>(&self, query: &str) -> Vec<T>
  where
    T: for<'de> serde::Deserialize<'de>,
  {
    log::trace!("get_rows_from_database: query = {}", query);
    let mut statement = self.connection.prepare(query).unwrap();
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

  pub fn migrate(&self, path: &str) {
    let paths = std::fs::read_dir(path).unwrap();
    for path in paths {
      let path = path.unwrap().path();
      let sql = std::fs::read_to_string(path).unwrap();
      self.execute_query(&sql, ()).unwrap();
    }
  }

  pub fn insert<T: ToQuery>(&self, row: &T) -> Result<usize, rusqlite::Error> {
    let (query, params) = row.insert();
    return self.execute_query(query, params.as_slice());
  }

  pub fn batch_insert<T: ToQuery>(&mut self, rows: &Vec<T>) -> Result<usize, rusqlite::Error> {
    let transaction = self.connection.transaction()?;
    let mut num_rows_affected = 0;
    for row in rows {
      let (query, parameters) = row.insert();
      num_rows_affected += transaction.execute(query, parameters.as_slice())?;
    }
    transaction.commit()?;
    return Ok(num_rows_affected);
  }
}
