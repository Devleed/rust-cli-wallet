use ethers::prelude::*;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::H160;
use lazy_static::lazy_static;
use reqwest;
use serde::Deserialize;
use std::sync::Mutex;

use crate::networks;
use crate::utils::log;
use crate::utils::perform_selection;
use crate::utils::LogSeverity;

lazy_static! {
    static ref PROVIDER: Mutex<Provider<Http>> = {
        let network_url =
            networks::get_network_url_by_chain_id(&networks::DEFAULT_SELECTED_CHAINID);
        Mutex::new(
            Provider::<Http>::try_from(network_url)
                .expect("Failed to connect to provider, try again."),
        )
    };
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    message: String,
    result: ResultData,
}

#[derive(Debug, Deserialize)]
pub struct ResultData {
    LastBlock: String,
    SafeGasPrice: String,
    ProposeGasPrice: String,
    FastGasPrice: String,
    suggestBaseFee: String,
    gasUsedRatio: String,
}

pub fn set_provider(network_url: &str) {
    let mut data = PROVIDER.lock().unwrap();

    let provider =
        Provider::<Http>::try_from(network_url).expect("Failed to connect to provider, try again.");

    *data = provider;
}
pub fn get_provider() -> Provider<Http> {
    PROVIDER.lock().unwrap().clone()
}
pub async fn fetch_balance(address: H160) -> Result<f64, Box<dyn std::error::Error>> {
    let provider = PROVIDER.lock().unwrap();

    let balance = provider
        .get_balance(address, None)
        .await
        .expect("Failed to fetch user balance");

    Ok(ethers::utils::format_units(balance, "ether")?
        .trim()
        .parse::<f64>()?)
}

pub async fn estimate_gas(tx: &TypedTransaction, gas_price: Option<U256>) -> String {
    log(
        "Transaction cost is not accurate for Goerli",
        Some(LogSeverity::WARN),
    );

    let provider = PROVIDER.lock().unwrap();

    let provider_gas_price: U256 = if gas_price.is_none() {
        provider
            .get_gas_price()
            .await
            .expect("Error fetching gas price.")
    } else {
        gas_price.unwrap()
    };

    let gas = provider
        .estimate_gas(tx, None)
        .await
        .expect("Failed to estimate gas.");

    return ethers::utils::format_ether(gas * provider_gas_price);
}

pub async fn get_network_gas_prices() -> Result<ResultData, &'static str> {
    let url = "https://api.etherscan.io/api?module=gastracker&action=gasoracle&apikey=3MB1GI7WAG539CW1NKSHW22I7C2WH8HGB8";

    // Make a GET request and await the response
    let response = reqwest::get(url).await.expect("Failed on api call");

    // Check if the request was successful (status code 2xx)
    if response.status().is_success() {
        // Read the response body as a string
        let body: ApiResponse = response.json().await.expect("Failed to parse");

        return Ok(body.result);
    } else {
        // Print an error message if the request was not successful
        eprintln!("Request failed with status code: {}", response.status());
        return Err("Unable to fetch network gas prices.");
    }
}

pub async fn gas_price_selector() -> U256 {
    let gas_prices = get_network_gas_prices().await.unwrap();

    let mut slow_option = String::from("Slow ");
    slow_option.push_str(&gas_prices.SafeGasPrice);

    let mut medium_option = String::from("Medium ");
    medium_option.push_str(&gas_prices.ProposeGasPrice);

    let mut fast_option = String::from("Fast ");
    fast_option.push_str(&gas_prices.FastGasPrice);

    let mut view_gas_options = vec![slow_option, medium_option, fast_option];
    let gas_options = vec![
        &gas_prices.SafeGasPrice,
        &gas_prices.ProposeGasPrice,
        &gas_prices.FastGasPrice,
    ];

    let selection = perform_selection(
        "Select gas price",
        &mut view_gas_options,
        Some("Select gas price"),
        false,
    );

    let selected_gas_price = &gas_options[selection.unwrap()]
        .parse::<U256>()
        .expect("Error parsing gas price.");

    return selected_gas_price.clone();
}
