use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::types::Address;
use serde::{Deserialize, Serialize};

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

pub fn launch_token_actions(token: &Token) {
    loop {
        let actions = vec!["Display balance", "Send"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&actions)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .expect("Failed to create token action list.");
    }
}
