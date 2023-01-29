use black_scholes::PricesAndGreeks;
use chrono::DateTime;
use chrono_tz::Tz;

pub fn calculate_greeks(now: DateTime<Tz>, expiration_date: DateTime<Tz>, underlying_price: f64, strike_price: f64, option_iv: f64, risk_free_rate: f64) -> PricesAndGreeks {
    let minutes_to_expiration = expiration_date.signed_duration_since(now).num_minutes();
    let minutes_per_day = 60.0 * 24.0;
    let days_to_expiration = (minutes_to_expiration as f64) / minutes_per_day;
    let maturity = days_to_expiration / 365.0; // TODO: should this be / 252 because of 252 trading days a year?
                                               // TODO: does this work because SPY = american style options but SPX = european style?
                                               // TODO: dividend yield?
    return black_scholes::compute_all(underlying_price, strike_price, risk_free_rate, option_iv, maturity);
}
