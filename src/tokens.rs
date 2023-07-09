use ethers::prelude::*;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::types::{Address, H160};
use serde::{Deserialize, Serialize};

use crate::wallet;
use crate::{account, ierc20::IERC20, networks, provider, utils};
use std::{fs, sync::Arc};

#[derive(Serialize, Deserialize, Clone)]
pub struct Token {
    pub name: String,
    pub decimals: u8,
    pub address: Address,
    pub chain_id: u8,
}

pub async fn add_token() {
    let chain_id = networks::get_selected_chain_id();
    let account_name = account::get_account_name().unwrap();

    let mut token_address = String::from("");
    utils::take_user_input("Token address", &mut token_address, "Enter token address");

    let address: Address = token_address.trim().parse().unwrap();

    let contract = create_contract_instance(address);
    let name = contract
        .name()
        .await
        .expect("Token of provided address on selected network does not exist");
    let decimals = contract
        .decimals()
        .await
        .expect("Token of provided address on selected network does not exist");

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/tokens");

    let token = Token {
        name,
        decimals,
        address,
        chain_id,
    };

    let token_json = serde_json::to_string(&token).unwrap();

    let mut token_path = String::from(&account_path);
    token_path.push_str("/");
    token_path.push_str(token.address.to_string().as_str());

    fs::create_dir_all(&account_path).unwrap();
    fs::write(token_path, token_json.as_bytes()).expect("Failed to add token");
}

pub fn get_user_tokens() -> Vec<Token> {
    let chain_id = networks::get_selected_chain_id();
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/tokens");

    let tokens = fs::read_dir(&account_path).expect("Failed to read directory");

    tokens
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().into_string().unwrap())
        .filter_map(|token_address| {
            let mut token_path = String::from(&account_path);
            token_path.push_str("/");
            token_path.push_str(&token_address);

            let token_json = fs::read_to_string(&token_path).unwrap();
            let token: Token = serde_json::from_str(token_json.trim()).unwrap();

            if token.chain_id == chain_id {
                Some(token)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
pub async fn fetch_token_balance(token_address: H160, user_address: H160) -> f64 {
    let contract = create_contract_instance(token_address);

    let balance = contract
        .balance_of(user_address)
        .await
        .expect("Failed to fetch user token balance");

    ethers::utils::format_units(balance, "ether")
        .unwrap()
        .trim()
        .parse::<f64>()
        .unwrap()
}
pub async fn send_token(token: &Token) {
    let provider = provider::get_provider();
    let wallet = wallet::get_wallet().unwrap();
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let contract = IERC20::new(token.address, client);

    let mut recipient = String::new();
    utils::take_user_input("Recipient", &mut recipient, "Enter recipient");
    let to_address: Address = recipient.trim().parse().unwrap();

    let mut value = String::new();
    utils::take_user_input("value", &mut value, "Enter value");
    let value: u64 = value.trim().parse().unwrap();
    let decimal_amount = U256::from(value) * U256::exp10(token.decimals as usize);

    let tx = contract.transfer(to_address, decimal_amount);
    let pending_tx = tx.send().await.unwrap();
    let receipt = pending_tx.await.unwrap();

    if receipt.is_some() {
        println!("Tx hash: {:?}", receipt.unwrap().transaction_hash);
    }
}

/* PRIVATE FUNTIONS */
fn create_contract_instance(token_address: H160) -> IERC20<Provider<Http>> {
    let provider = provider::get_provider();
    let client = Arc::new(provider);

    let contract = IERC20::new(token_address, client);

    return contract;
}
