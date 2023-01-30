use log::warn;
use std::future::Future;

pub async fn timeout_wrapper<Fut, T>(timeout_ms: u64, cb: &impl Fn() -> Fut) -> Result<T, String>
where
  Fut: Future<Output = Result<T, String>>,
{
  let request_future = cb();
  let timeout_future = tokio::time::timeout(tokio::time::Duration::from_millis(timeout_ms), request_future).await;
  if timeout_future.is_err() {
    return Err(String::from("timed out"));
  }
  let response = timeout_future.unwrap();
  return response;
}

pub async fn retry_wrapper<Fut, T>(known_errors: &[&str], retry_delay_ms: u64, num_retries: usize, cb: &impl Fn() -> Fut) -> Result<T, String>
where
  Fut: Future<Output = Result<T, String>>,
{
  for attempt in 0..num_retries {
    let result = cb().await;
    let is_success = result.is_ok();
    if is_success {
      return Ok(result.unwrap());
    }
    let error_message = result.err().unwrap();
    let is_error_known = known_errors.iter().cloned().position(|known_error| {
      return error_message.contains(&known_error);
    });
    if is_error_known.is_none() {
      return Err(format!("unknown error: {}", error_message));
    }
    warn!("retry # {} / {}: {}", attempt, num_retries, error_message);
    // sleep
    tokio::time::sleep(tokio::time::Duration::from_millis(retry_delay_ms)).await;
  }
  return Err(format!("request failed after {} retries", num_retries));
}

pub async fn retry_timeout_wrapper<Fut, T>(
  known_errors: &[&str],
  retry_delay_ms: u64,
  num_retries: usize,
  timeout_ms: u64,
  cb: impl Fn() -> Fut,
) -> Result<T, String>
where
  Fut: Future<Output = Result<T, String>>,
{
  let timeout_cb = || {
    return timeout_wrapper(timeout_ms, &cb);
  };
  return retry_wrapper(known_errors, retry_delay_ms, num_retries, &timeout_cb).await;
}
