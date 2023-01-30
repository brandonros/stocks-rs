use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinnhubStockCandlesResponse {
  pub c: Vec<f64>,
  pub h: Vec<f64>,
  pub l: Vec<f64>,
  pub o: Vec<f64>,
  pub t: Vec<i64>,
  pub v: Vec<f64>,
}
