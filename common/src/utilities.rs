use rust_decimal::Decimal;

pub async fn aligned_sleep(amount_ms: i64) {
  let now = chrono::Utc::now();
  let difference = amount_ms - (now.timestamp_millis() % amount_ms);
  tokio::time::sleep(tokio::time::Duration::from_millis(difference as u64)).await;
}

pub fn build_decimal_range(min: Decimal, max: Decimal, step: Decimal) -> Vec<Decimal> {
  let mut pointer = min;
  let mut results = vec![];
  while pointer <= max {
    results.push(pointer);
    pointer += step;
  }
  return results;
}

pub fn build_usize_range(min: usize, max: usize, step: usize) -> Vec<usize> {
  let mut pointer = min;
  let mut results = vec![];
  while pointer <= max {
    results.push(pointer);
    pointer += step;
  }
  return results;
}
