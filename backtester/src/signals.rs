use crate::structs::{Direction, DirectionChange, SignalSnapshot};

pub fn build_direction_changes_from_signal_snapshots(signal_snapshots: &Vec<SignalSnapshot>, warmed_up_index: usize) -> Vec<DirectionChange> {
  let mut trade_direction = Direction::Flat;
  let mut direction_changes: Vec<DirectionChange> = vec![];
  for i in warmed_up_index..signal_snapshots.len() {
    let current_direction = (&signal_snapshots[i].direction).to_owned();
    if current_direction != trade_direction {
      // close any open trades
      if direction_changes.len() != 0 {
        let last_direction_change_index = direction_changes.len() - 1;
        let mut last_direction_change = &mut direction_changes[last_direction_change_index];
        last_direction_change.end_snapshot_index = Some(i);
      }
      // open new trade
      direction_changes.push(DirectionChange {
        start_snapshot_index: i,
        end_snapshot_index: None,
      });
      trade_direction = current_direction;
    }
  }
  // make sure last trade is closed
  let last_direction_change_index = direction_changes.len() - 1;
  let mut last_direction_change = &mut direction_changes[last_direction_change_index];
  last_direction_change.end_snapshot_index = Some(signal_snapshots.len() - 1);
  // return
  return direction_changes;
}
