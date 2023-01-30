use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolygonResult {
  pub c: f64,
  pub h: f64,
  pub l: f64,
  pub n: Option<i64>,
  pub o: f64,
  pub t: i64,
  pub v: f64,
  pub vw: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolygonResponseRoot {
  pub adjusted: bool,
  #[serde(rename = "queryCount")]
  pub query_count: i64,
  pub request_id: String,
  pub results: Vec<PolygonResult>,
  #[serde(rename = "resultsCount")]
  pub results_count: i64,
  pub status: String,
  pub ticker: String,
}
