#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use serde::{Deserialize};

#[derive(Deserialize)]
struct PolygonResult {
  pub c: f64,
  pub h: f64,
  pub l: f64,
  pub n: Option<i64>,
  pub o: f64,
  pub t: i64,
  pub v: f64,
  pub vw: Option<f64>,
}

#[derive(Deserialize)]
struct PolygonResponseRoot {
  pub adjusted: bool,
  #[serde(rename = "queryCount")]
  pub query_count: i64,
  pub request_id: String,
  pub results: Vec<PolygonResult>,
  #[serde(rename = "resultsCount")]
  pub results_count: i64,
  pub status: String,
  pub ticker: String,
}

fn main() {
    println!("start_timestamp,end_timestamp,open,high,low,close,volume");
    let mut dir_entries: Vec<_> = std::fs::read_dir("./data").unwrap().map(|r| r.unwrap()).collect();
    dir_entries.sort_by_key(|dir| dir.path());
    for dir_entry in dir_entries {
        let stringified_value = std::fs::read_to_string(&dir_entry.path()).unwrap();
        let parsed_value: PolygonResponseRoot = serde_json::from_str(&stringified_value).unwrap();
        for result in &parsed_value.results {
            let start_timestamp = result.t / 1000;
            let end_timestamp = start_timestamp + 300 - 1;
            let open = result.o;
            let high = result.h;
            let low = result.l;
            let close = result.c;
            let volume = result.v as i64;
            println!("{start_timestamp},{end_timestamp},{open},{high},{low},{close},{volume}");
        }
    }
}
