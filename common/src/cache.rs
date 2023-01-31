use std::collections::HashMap;

pub fn get<'a, T>(cache_map: &'a mut HashMap<&'a str, &'a T>, cache_key: &'a str, populate_fn: impl Fn() -> &'a T) -> &'a T {
  let value = cache_map.get(cache_key);
  if value.is_some() {
    return value.unwrap();
  }
  let value = populate_fn();
  cache_map.insert(cache_key, value);
  return &value;
}
