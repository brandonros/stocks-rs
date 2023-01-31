use std::collections::HashMap;

pub fn get<T>(cache_map: &HashMap<&str, T>, cache_key: &str, populate_fn: &impl Fn() -> &T) -> &T {
  let value = cache_map.get(cache_key);
  if value.is_some() {
    return value.unwrap();
  }
  let value = populate_fn();
  cache_map.insert(cache_key, value);
  return &value;
}
