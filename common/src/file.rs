use tokio::io::AsyncWriteExt;

pub async fn write_text_to_file(filename: &str, value: &String)
{
  let mut file = tokio::fs::File::create(filename).await.unwrap();
  file.write_all(value.as_bytes()).await.unwrap();
}

pub async fn write_json_to_file<T>(filename: &str, value: &T)
where
  T: serde::Serialize,
{
  let stringified_value = serde_json::to_string_pretty(value).unwrap();
  let mut file = tokio::fs::File::create(filename).await.unwrap();
  file.write_all(stringified_value.as_bytes()).await.unwrap();
}

pub async fn read_text_from_file(filename: &str) -> String
{
  return tokio::fs::read_to_string(filename).await.unwrap();
}

pub async fn read_json_from_file<T>(filename: &str) -> T
where
  T: for<'de> serde::Deserialize<'de>,
{
  let stringified_value = tokio::fs::read_to_string(filename).await.unwrap();
  return serde_json::from_str(&stringified_value).unwrap();
}

pub fn sync_read_json_from_file<T>(filename: &str) -> T
where
  T: for<'de> serde::Deserialize<'de>,
{
  let stringified_value = std::fs::read_to_string(filename);
  if stringified_value.is_err() {
    panic!("error reading {filename}");
  }
  let stringified_value = stringified_value.unwrap();
  let parsed_value = serde_json::from_str(&stringified_value);
  if parsed_value.is_err() {
    panic!("error parsing {filename}");
  }
  return parsed_value.unwrap();
}
