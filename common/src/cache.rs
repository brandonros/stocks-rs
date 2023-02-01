use std::{collections::HashMap, rc::Rc};

pub fn get<T>(cache_map: &mut HashMap<String, Rc<T>>, cache_key: &str, populate_fn: &impl Fn() -> T) -> Rc<T> {
  match cache_map.get(cache_key) {
    Some(value) => value.clone(),
    None => {
      let value = Rc::new(populate_fn());
      cache_map.insert(cache_key.to_string(), value.clone());
      value
    }
  }
}
