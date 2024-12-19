use std::{fs, path::Path};

mod account;
mod beneficiaries;
mod fiat;
mod ierc20;
mod keystore;
mod networks;
mod provider;
mod tokens;
mod utils;
mod wallet;

#[tokio::main]
async fn main() {
    if !Path::new("accounts").exists() {
        // accounts directory does not exists, create it first
        fs::create_dir("accounts").expect("Failed to create accounts directory");
    }

    // Load content from abi/erc20.rs at compile time
    let _erc20_abi = include_str!("../abis/erc20.json");

    // Load content from config/chains.json at compile time
    let _chains_config = include_str!("../config/chains.json");

    loop {
        print!("inside main loop!. ");

        account::launch_app().await;
    }
}
