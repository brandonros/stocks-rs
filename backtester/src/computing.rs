use common::{database::Database, structs::Candle};
use providers::*;
use strategies::{supertrend::*, vwap_mvwap_ema_crossover::*, *};

use crate::{market_session, signals};

pub async fn compute(symbol: &str, resolution: &str, provider: &Provider, strategy: &Strategy, date: &str) {
  // connect to databse
  let connection = Database::new(&format!("{:?}", provider));
  connection.migrate("./schema/");
  // load indicator settings
  let warmed_up_index = match strategy {
    Strategy::Supertrend => 10,
    Strategy::VwapMvwapEmaCrossover => 25,
  };
  let indicator_settings = match strategy {
    Strategy::Supertrend => StrategyIndicatorSettings::Supertrend(SupertrendStrategyIndicatorSettings {
      supertrend_periods: 10,
      supertrend_multiplier: 3.0,
    }),
    Strategy::VwapMvwapEmaCrossover => StrategyIndicatorSettings::VwapMvwapEmaCrossover(VwapMvwapEmaCrossoverStrategyIndicatorSettings {
      vwap_ema_fast_periods: 1,
      vwap_ema_slow_periods: 21,
      ema_fast_periods: 7,
      ema_slow_periods: 25,
    }),
  };
  // pull candles
  let (from, to) = market_session::get_regular_market_start_end_from_string(date);
  let from_timestamp = from.timestamp();
  let to_timestamp = to.timestamp();
  let query = format!(
    "SELECT * FROM candles WHERE resolution = '{resolution}' AND symbol = '{symbol}' AND timestamp >= {from_timestamp} AND timestamp <= {to_timestamp}"
  );
  let candles = connection.get_rows_from_database::<Candle>(&query);
  // build snapshots from candles
  let signal_snapshots = match strategy {
    Strategy::Supertrend => {
      let strategy = SupertrendStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::Supertrend(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles)
    }
    Strategy::VwapMvwapEmaCrossover => {
      let strategy = VwapMvwapEmaCrossoverStrategy::new();
      let indicator_settings = match indicator_settings {
        StrategyIndicatorSettings::VwapMvwapEmaCrossover(indicator_settings) => indicator_settings,
        _ => unreachable!(),
      };
      strategy.build_signal_snapshots_from_candles(&indicator_settings, &candles)
    }
  };
  // calculate direction changes
  let direction_changes = signals::build_direction_changes_from_signal_snapshots(&signal_snapshots, warmed_up_index);
  // dump latest direction change
  if direction_changes.len() == 0 {
    log::warn!("no direction changes yet");
    return;
  }
  let most_recent_direction_change = &direction_changes[direction_changes.len() - 1];
  let start_snapshot = &signal_snapshots[most_recent_direction_change.start_snapshot_index];
  log::info!("{:?}", most_recent_direction_change);
  log::info!("{:?}", start_snapshot);
}
