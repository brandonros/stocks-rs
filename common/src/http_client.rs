use std::str::FromStr;

use crate::retry;
use reqwest::{
  header::{HeaderMap, HeaderName, HeaderValue},
  Client, Method,
};

pub async fn http_request_text(
  http_client: &Client,
  method_str: &str,
  url: &str,
  request_headers: &Vec<(String, String)>,
  payload: &Option<String>,
) -> Result<(HeaderMap, String), String> {
  log::info!("http_request_text: method = {} url = {}", method_str, url);
  let mut request_headers_map = HeaderMap::new();
  for (key, value) in request_headers {
    request_headers_map.insert(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value).unwrap());
  }
  let method = Method::from_bytes(method_str.as_bytes()).unwrap();
  let request = if payload.is_some() {
    let payload = payload.as_ref().unwrap();
    http_client.request(method, url).headers(request_headers_map).body(payload.to_owned())
  } else {
    http_client.request(method, url).headers(request_headers_map)
  };
  let response = request.send().await;
  if response.is_err() {
    return Err(format!("{}", response.err().unwrap()));
  }
  let response = response.unwrap();
  let response_status = response.status().as_u16();
  let is_2xx = response_status >= 200 && response_status <= 299;
  if is_2xx == false {
    return Err(format!("invalid response status: {}", response.status().as_u16()));
  }
  let response_headers = response.headers().to_owned();
  let stringified_response_body = response.text().await.unwrap();
  log::debug!("stringified_response_body = {}", stringified_response_body);
  return Ok((response_headers, stringified_response_body));
}

pub async fn http_request_json<T>(
  http_client: &Client,
  method_str: &str,
  url: &str,
  headers: &Vec<(String, String)>,
  payload: &Option<String>,
) -> Result<T, String>
where
  T: for<'de> serde::Deserialize<'de>,
{
  log::info!("http_request_json: method = {} url = {}", method_str, url);
  let result = http_request_text(http_client, method_str, url, headers, payload).await;
  if result.is_err() {
    let err = result.unwrap_err();
    return Err(err);
  }
  let (_response_headers, stringified_response_body) = result.unwrap();
  let response_body: T = if stringified_response_body.len() == 0 {
    serde_json::from_str("null").unwrap() // watch out for empty response body
  } else {
    serde_json::from_str(&stringified_response_body).unwrap()
  };
  return Ok(response_body);
}

pub async fn http_request_json_with_timeout_and_retries<T>(
  http_client: &Client,
  method_str: &str,
  url: &str,
  headers: &Vec<(String, String)>,
  payload: &Option<String>,
  known_errors: &Vec<String>,
  timeout_ms: u64,
  retry_delay_ms: u64,
  num_retries: usize,
) -> Result<T, String>
where
  T: for<'de> serde::Deserialize<'de>,
{
  let cb = || {
    return http_request_json::<T>(http_client, method_str, url, &headers, &payload);
  };
  return retry::retry_timeout_wrapper(known_errors, retry_delay_ms, num_retries, timeout_ms, &cb).await;
}

pub async fn http_request_text_with_timeout_and_retries(
  http_client: &Client,
  method_str: &str,
  url: &str,
  headers: &Vec<(String, String)>,
  payload: &Option<String>,
  known_errors: &Vec<String>,
  timeout_ms: u64,
  retry_delay_ms: u64,
  num_retries: usize,
) -> Result<(HeaderMap, String), String> {
  let cb = || {
    return http_request_text(http_client, method_str, url, &headers, &payload);
  };
  return retry::retry_timeout_wrapper(known_errors, retry_delay_ms, num_retries, timeout_ms, &cb).await;
}
