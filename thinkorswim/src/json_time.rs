use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S: Serializer>(time: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error> {
    DateTime::<Utc>::from_utc(time.to_owned(), Utc).to_rfc3339().serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDateTime, D::Error> {
    let time: String = Deserialize::deserialize(deserializer).unwrap();
    Ok(DateTime::parse_from_rfc3339(&time).map_err(D::Error::custom)?.naive_utc())
}
