use crate::networks::get_selected_chain_coin;
use crate::provider::gas_price_selector;
use crate::utils::{is_valid_ethereum_address, launch_tx_thread, log_tx};
use crate::{beneficiaries, fiat, provider, utils};
use coins_bip32::prelude::SigningKey;
use ethers::prelude::*;
use ethers::{
    prelude::SignerMiddleware,
    signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Signer, Wallet},
    types::{transaction::eip2718::TypedTransaction, TransactionReceipt, TransactionRequest},
};
use lazy_static::lazy_static;
use std::ops::Mul;
use std::sync::Mutex;

lazy_static! {
    static ref WALLET: Mutex<Option<Wallet<SigningKey>>> = Mutex::new(None);
}

pub fn build_wallet(account_key: &str, chain_id: u32) {
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

    set_wallet(Some(wallet));
}
pub fn get_wallet() -> Option<Wallet<SigningKey>> {
    let wallet = WALLET.lock().unwrap();

    wallet.clone()
}
pub fn set_wallet(wallet: Option<Wallet<SigningKey>>) {
    let mut data = WALLET.lock().unwrap();
    *data = wallet;
}
pub async fn send_eth() -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let wallet = get_wallet().unwrap();

    let provider = provider::get_provider();
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = wallet.address();

    let balance_from = provider::fetch_balance(address_from).await?;

    println!("Available balance: {}", balance_from);

    let mut send_options = vec!["select beneficiary".to_string(), "type address".to_string()];

    let selected_option = utils::perform_selection("Send options", &mut send_options, None, true);

    let address_to = if selected_option.is_some() {
        if selected_option.unwrap() == 0 {
            // select beneficiary
            beneficiaries::select_beneficiary()
        } else {
            // take address input
            Some(
                utils::take_user_input(
                    "Address",
                    "Type recipient address",
                    Some(is_valid_ethereum_address),
                )
                .trim()
                .parse::<H160>()
                .unwrap(),
            )
        }
    } else {
        None
    };

    if address_to.is_none() {
        return Ok(None);
    }

    let mut value_str = utils::take_user_input("value", "Enter amount to send:", None);

    let mut transaction_req: TypedTransaction = TransactionRequest::new()
        .from(address_from)
        .to(address_to.unwrap())
        .value(U256::from(ethers::utils::parse_ether(value_str.trim())?))
        .into();

    let selected_gas_price = gas_price_selector().await;

    let tx_cost = provider::estimate_gas(&mut transaction_req, Some(selected_gas_price)).await;

    let mut parsed_val = value_str.trim().parse::<f64>()? + tx_cost;

    while parsed_val.ge(&balance_from) {
        println!(
            "Amount limit exceeded, sender has {} ETH and you're trying to send {} ETH with {} ETH tx cost. \n",
            balance_from,
            value_str.trim(),
            parsed_val
        );
        value_str = utils::take_user_input("value", "Enter amount to send in ETH:", None);
        parsed_val = value_str.trim().parse::<f64>()? + tx_cost;
    }

    let selected_token = get_selected_chain_coin();
    let fiat_rate = fiat::get_fiat_rate();

    println!(
        "\nSending {} {} ({} USD) from {:?} to {:?}\n",
        parsed_val,
        selected_token,
        parsed_val.mul(fiat_rate),
        address_from,
        address_to.unwrap()
    );

    let tx_confirmation = utils::take_user_input(
        "confirmation",
        "Are you sure you want to perform this transaction? [Y/N]",
        None,
    );

    if tx_confirmation.trim().to_lowercase() == "y" {
        launch_tx_thread(async move {
            let pending_tx = client
                .send_transaction(transaction_req, None)
                .await
                .unwrap();

            println!("Pending tx hash: {:?}", pending_tx.tx_hash());

            let sent_tx = pending_tx.await.unwrap();

            let receipt = sent_tx.expect("failed to send transaction");

            log_tx(Some(receipt))
        });

        Ok(None)
    } else {
        Ok(None)
    }
}

pub async fn create_dummy_send_tx() -> Option<TypedTransaction> {
    let wallet = get_wallet();

    if let Some(selected_wallet) = wallet {
        let transaction_req: TypedTransaction = TransactionRequest::new()
            .from(selected_wallet.address())
            .to(selected_wallet.address())
            .value(U256::from("10000000000"))
            .into();

        return Some(transaction_req);
    } else {
        None
    }
}
