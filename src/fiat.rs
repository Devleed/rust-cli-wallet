use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Mutex;

use crate::utils::{fetch_and_parse, log, LogSeverity};

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct ApiResponse {
    USD: f64,
}

lazy_static! {
    static ref CURRENCY_RATE: Mutex<f64> = Mutex::new(1.0);
}

pub async fn set_fiat_rate(selected_token: &str) {
    match fetch_fiat_rate(selected_token).await {
        Ok(_) => {}
        Err(err) => {
            log(&err, Some(LogSeverity::ERROR));
        }
    }
}

pub async fn fetch_fiat_rate(selected_token: &str) -> Result<(), String> {
    let url = format!(
        "https://min-api.cryptocompare.com/data/price?fsym={}&tsyms=USD",
        selected_token
    );

    let body = fetch_and_parse::<ApiResponse>(&url).await?;

    // Update the CURRENCY_RATE global safely
    let mut data = CURRENCY_RATE
        .lock()
        .map_err(|_| "Failed to lock mutex".to_string())?;
    *data = body.USD;

    Ok(())
}

pub fn get_fiat_rate() -> f64 {
    CURRENCY_RATE.lock().unwrap().clone()
}
