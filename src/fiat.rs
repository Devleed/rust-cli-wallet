use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct ApiResponse {
    USD: f64,
}

lazy_static! {
    static ref CURRENCY_RATE: Mutex<f64> = Mutex::new(1.0);
}

pub async fn set_fiat_rate(selected_token: &str) {
    let url = format!(
        "https://min-api.cryptocompare.com/data/price?fsym={}&tsyms=USD",
        selected_token
    );

    // Make a GET request and await the response
    let response = reqwest::get(url).await.expect("Failed on api call");

    // Check if the request was successful (status code 2xx)
    if response.status().is_success() {
        // Read the response body as a string
        let body: ApiResponse = response.json().await.expect("Failed to parse");

        let mut data = CURRENCY_RATE.lock().unwrap();
        *data = body.USD;
    } else {
        // Print an error message if the request was not successful
        eprintln!("Request failed with status code: {}", response.status());
    }
}

pub fn get_fiat_rate() -> f64 {
    CURRENCY_RATE.lock().unwrap().clone()
}
