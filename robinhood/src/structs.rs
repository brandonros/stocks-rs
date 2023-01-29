use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CashComponent {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnderlyingInstrument {
  pub id: String,
  pub instrument: String,
  pub quantity: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MinTicks {
  pub above_tick: String,
  pub below_tick: String,
  pub cutoff_price: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chain {
  pub id: String,
  pub symbol: String,
  pub can_open_position: bool,
  pub cash_component: Option<CashComponent>,
  pub expiration_dates: Vec<String>,
  pub trade_value_multiplier: String,
  pub underlying_instruments: Vec<UnderlyingInstrument>,
  pub min_ticks: MinTicks,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quote {
  pub ask_price: String,
  pub ask_size: u32,
  pub bid_price: String,
  pub bid_size: u32,
  pub last_trade_price: String,
  pub last_extended_hours_trade_price: Option<String>,
  pub last_non_reg_trade_price: Option<String>,
  pub previous_close: String,
  pub adjusted_previous_close: String,
  pub previous_close_date: String,
  pub symbol: String,
  pub trading_halted: bool,
  pub has_traded: bool,
  pub last_trade_price_source: String,
  pub last_non_reg_trade_price_source: String,
  pub updated_at: String,
  pub instrument: String,
  pub instrument_id: String,
  pub state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionInstrument {
  pub chain_id: String,
  pub chain_symbol: String,
  pub created_at: String,
  pub expiration_date: String,
  pub id: String,
  pub issue_date: String,
  pub min_ticks: MinTicks,
  pub rhs_tradability: String,
  pub tradability_for_late_closing_etfs: Option<String>,
  pub state: String,
  pub strike_price: String,
  pub tradability: String,
  pub r#type: String,
  pub updated_at: String,
  pub url: String,
  pub sellout_datetime: String,
  pub long_strategy_code: String,
  pub short_strategy_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionsInstrumentsResults {
  pub results: Vec<OptionInstrument>,
  pub next: Option<String>,
  pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionMarketData {
  pub adjusted_mark_price: String,
  pub adjusted_mark_price_round_down: String,
  pub ask_price: String,
  pub ask_size: u32,
  pub bid_price: String,
  pub bid_size: u32,
  pub break_even_price: String,
  pub high_price: Option<String>,
  pub instrument: String,
  pub instrument_id: String,
  pub last_trade_price: Option<String>,
  pub last_trade_size: Option<u32>,
  pub low_price: Option<String>,
  pub mark_price: String,
  pub open_interest: u32,
  pub previous_close_date: String,
  pub previous_close_price: String,
  pub updated_at: String,
  pub volume: u32,
  pub symbol: String,
  pub occ_symbol: String,
  pub state: String,
  pub chance_of_profit_long: Option<String>,
  pub chance_of_profit_short: Option<String>,
  pub delta: Option<String>,
  pub gamma: Option<String>,
  pub implied_volatility: Option<String>,
  pub rho: Option<String>,
  pub theta: Option<String>,
  pub vega: Option<String>,
  pub high_fill_rate_buy_price: Option<String>,
  pub high_fill_rate_sell_price: Option<String>,
  pub low_fill_rate_buy_price: Option<String>,
  pub low_fill_rate_sell_price: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionsMarketDataResult {
  pub results: Vec<Option<OptionMarketData>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionSeries {
  pub options: Vec<OptionInstrument>,
  pub options_quotes: Vec<OptionMarketData>,
}
