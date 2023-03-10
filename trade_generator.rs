#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! csv = "1.2.1"
//! serde = { version = "1.0.153", features = ["derive"] }
//! ```

use std::fs::File;
use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(PartialEq, Debug, Deserialize, Clone)]
enum Direction {
  Long,
  Short,
  Flat,
}

enum Action {
  NoChange,
  Close,
  OpenNew,
  SwitchDirection
}

#[derive(Deserialize)]
struct Signal {
  pub start_timestamp: i64,
  pub end_timestamp: i64,
  pub direction: Direction
}

fn read_records_from_csv<T>(filename: &str) -> Vec<T>
where
  T: for<'de> Deserialize<'de>{
  let mut candles = vec![];
  let file = File::open(filename).unwrap();
  let mut csv_reader = ReaderBuilder::new()
    .has_headers(true)
    .from_reader(file);
  for record in csv_reader.deserialize() {
    let candle: T = record.unwrap();
    candles.push(candle);
  }
  return candles;
}

fn main() {
  // load signals
  let signals_filename = std::env::args().nth(1).unwrap();
  let signals = read_records_from_csv::<Signal>(&signals_filename);
  // print header
  println!("timestamp,type,direction");
  // convert signals to trades
  let mut last_direction = Direction::Flat;
  for signal in &signals {
    let signal_direction = &signal.direction;
    let action = match (&last_direction, signal_direction) {
      // stay in (no change)
      (Direction::Short, Direction::Short) => {
        Action::NoChange
      }
      (Direction::Long, Direction::Long) => {
        Action::NoChange
      }
      (Direction::Flat, Direction::Flat) => {
        Action::NoChange
      }
      // get out (close)
      (Direction::Long, Direction::Flat) => {
        Action::Close
      }
      (Direction::Short, Direction::Flat) => {
        Action::Close
      }
      // open new
      (Direction::Flat, Direction::Long) => {
        Action::OpenNew
      }
      (Direction::Flat, Direction::Short) => {
        Action::OpenNew
      }
      // switch direction
      (Direction::Short, Direction::Long) => {
        Action::SwitchDirection
      }
      (Direction::Long, Direction::Short) => {
        Action::SwitchDirection
      }
    };
    match action {
      Action::OpenNew => {
        println!("{timestamp},Open,{direction:?}", timestamp = signal.start_timestamp, direction = signal.direction);
      }
      Action::NoChange => {
        
      }
      Action::Close => {
        println!("{timestamp},Close,{direction:?}", timestamp = signal.start_timestamp, direction = last_direction);
      }
      Action::SwitchDirection => {
        println!("{timestamp},Close,{direction:?}", timestamp = signal.start_timestamp, direction = last_direction);
        println!("{timestamp},Open,{direction:?}", timestamp = signal.start_timestamp, direction = signal.direction);
      }
    }
    last_direction = signal.direction.clone();
  }
}
