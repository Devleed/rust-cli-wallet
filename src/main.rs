use std::{fs, path::Path};

mod account;
mod beneficiaries;
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

    loop {
        account::launch_app().await;
    }
}
