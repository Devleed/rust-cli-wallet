use ethers::types::Address;
use serde::{Deserialize, Serialize};

use crate::{account::get_account_name, ierc20::IERC20, networks, provider, utils};
use std::{fs, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct Token {
    name: String,
    decimals: u8,
    address: Address,
    chain_id: u8,
}

pub async fn add_token() {
    let chain_id = networks::get_selected_chain_id();

    let mut token_address = String::from("");
    utils::take_user_input("Token address", &mut token_address, "Enter token address");

    let provider = provider::get_provider();
    let client = Arc::new(provider);
    let address: Address = token_address.trim().parse().unwrap();

    let contract = IERC20::new(address, client);
    let name = contract
        .name()
        .await
        .expect("Token of provided address on selected network does not exist");
    let decimals = contract
        .decimals()
        .await
        .expect("Token of provided address on selected network does not exist");

    let account_name = get_account_name().unwrap();

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
