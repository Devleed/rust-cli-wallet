use ethers::prelude::*;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::H160;
use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::networks;
use crate::utils::log;
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

pub async fn estimate_gas(tx: &TypedTransaction) -> String {
    log(
        "Transaction cost is not accurate for Goerli",
        Some(LogSeverity::WARN),
    );

    let provider = PROVIDER.lock().unwrap();

    let gas_price = provider
        .get_gas_price()
        .await
        .expect("Error fetching gas price.");

    let gas = provider
        .estimate_gas(tx, None)
        .await
        .expect("Failed to estimate gas.");

    return ethers::utils::format_ether(gas * gas_price);
}
