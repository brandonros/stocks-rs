use std::str::FromStr;

use log::{debug, info};
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
  info!("http_request_text: method = {} url = {}", method_str, url);
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
  debug!("stringified_response_body = {}", stringified_response_body);
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
  info!("http_request_json: method = {} url = {}", method_str, url);
  let result = http_request_text(http_client, method_str, url, headers, payload).await;
  if result.is_err() {
    let err = result.unwrap_err();
    return Err(err);
  }
  let (_response_headers, stringified_response_body) = result.unwrap();
  // watch out for empty response body
  if stringified_response_body.len() == 0 {
    return Ok(serde_json::from_str("null").unwrap());
  }
  // try to parse response body
  let result = serde_json::from_str(&stringified_response_body);
  if result.is_err() {
    return Err(result.err().unwrap().to_string());
  }
  return Ok(result.unwrap());
}