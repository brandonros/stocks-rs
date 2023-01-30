pub async fn aligned_sleep(amount_ms: i64) {
  let now = chrono::Utc::now();
  let difference = amount_ms - (now.timestamp_millis() % amount_ms);
  tokio::time::sleep(tokio::time::Duration::from_millis(difference as u64)).await;
}
