use crate::{beneficiaries, provider, utils};
use coins_bip32::prelude::SigningKey;
use ethers::prelude::*;
use ethers::{
    prelude::SignerMiddleware,
    signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, TransactionReceipt, TransactionRequest},
};
use lazy_static::lazy_static;
use spinners::{Spinner, Spinners};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref WALLET: Mutex<Option<Wallet<SigningKey>>> = Mutex::new(None);
}

pub fn build_wallet(account_key: &str, chain_id: u8) {
    let wallet = if utils::is_pkey(account_key) {
        account_key
            .parse::<LocalWallet>()
            .expect("Error generating wallet from pkey")
            .with_chain_id(chain_id)
    } else {
        MnemonicBuilder::<English>::default()
            .phrase(account_key)
            .build()
            .expect("Error generating wallet from seed phrase")
            .with_chain_id(chain_id)
    };

    let mut data = WALLET.lock().unwrap();
    *data = Some(wallet);
}
pub fn get_wallet() -> Option<Wallet<SigningKey>> {
    let wallet = WALLET.lock().unwrap();

    wallet.clone()
}
pub async fn send_eth() -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let wallet = get_wallet().unwrap();
    let provider = provider::get_provider();
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = wallet.address();

    let balance_from = provider::fetch_balance(address_from).await?;
    let gas_price = provider::fetch_gas_price().await?;

    println!("Available balance: {}", balance_from);

    let address_to = beneficiaries::select_beneficiary().unwrap();
    let mut value_str = String::new();
    utils::take_user_input("value", &mut value_str, "\n\nEnter amount to send in ETH:");

    while value_str.trim().parse::<f64>()?.ge(&balance_from) {
        println!(
            "Amount limit exceeded, sender has {} ETH and you're trying to send {} ETH \n",
            balance_from,
            value_str.trim()
        );
        value_str = String::new();
        utils::take_user_input("value", &mut value_str, "Enter amount to send in ETH:");
    }

    let transaction_req: TypedTransaction = TransactionRequest::new()
        .from(address_from)
        .to(address_to)
        .value(U256::from(ethers::utils::parse_ether(value_str.trim())?))
        .into();

    let estimated_gas = provider
        .estimate_gas(&transaction_req, None)
        .await?
        .to_string()
        .parse::<f64>()?;

    let tx_cost = gas_price * estimated_gas;

    println!("tx cost: {} ETH", tx_cost);

    println!(
        "\nSending {} ETH from {:?} to {:?}\n",
        value_str.trim(),
        address_from,
        address_to
    );

    let mut tx_confirmation = String::new();
    utils::take_user_input(
        "confirmation",
        &mut tx_confirmation,
        "Are you sure you want to perform this transaction? [Y/N]",
    );

    if tx_confirmation.trim().to_lowercase() == "y" {
        let mut sp = Spinner::new(Spinners::Dots9, "Transaction pending".into());
        sleep(Duration::from_secs(3));
        sp.stop();

        let sent_tx = client
            .send_transaction(transaction_req, None)
            .await?
            .await?;

        let receipt = sent_tx.expect("failed to send transaction");

        println!("Tx hash: {:?}", receipt.transaction_hash);

        Ok(Some(receipt))
    } else {
        Ok(None)
    }
}
