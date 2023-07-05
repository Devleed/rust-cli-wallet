use std::{fs, sync::Mutex};

use coins_bip32::prelude::SigningKey;
use ethers::signers::{
    coins_bip39::{English, Mnemonic},
    LocalWallet, MnemonicBuilder, Signer, Wallet,
};
use lazy_static::lazy_static;

use crate::{keystore, networks, utils};

lazy_static! {
    static ref WALLET: Mutex<Option<Wallet<SigningKey>>> = Mutex::new(None);
    static ref ACCOUNT_KEY: Mutex<Option<String>> = Mutex::new(None);
}

pub fn import_wallet() {
    let secret = take_secret_input().unwrap();

    let (account_key, _account_name) = create_new_acc(Some(secret));

    build_wallet(&account_key, networks::get_selected_chain_id());
}

pub fn create_wallet() {
    let mut create_new_acc_confirmation = String::new();
    utils::take_user_input(
        "Confirmation",
        &mut create_new_acc_confirmation,
        "Do you want to create a new wallet? [Y/N]",
    );

    if create_new_acc_confirmation.trim().to_lowercase() == "y" {
        // * create new wallet
        let (account_key, _account_name) = create_new_acc(None);

        // * generate wallet from phrase
        build_wallet(&account_key, networks::get_selected_chain_id());
    }
}

pub fn select_wallet(acc_name: &str) {
    // * create file path
    let mut file_name = String::from("accounts/");
    file_name.push_str(acc_name);

    // * read file from given path
    let account_json = fs::read_to_string(file_name.trim()).expect("Failed to read account.");

    let mut password_string = String::new();
    utils::take_user_input("Password", &mut password_string, "Enter password:");

    let secret_key = keystore::deserialize_keystore(&account_json, password_string.trim());

    let mut data = ACCOUNT_KEY.lock().unwrap();
    *data = Some(secret_key.clone());

    build_wallet(&secret_key, networks::get_selected_chain_id());
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

pub fn get_account_key() -> Option<String> {
    let data = ACCOUNT_KEY.lock().expect("Failed to lock acc key");

    data.clone()
}

/* Private functions */
fn create_new_acc(secret: Option<String>) -> (String, String) {
    let mut password_string = String::new();
    utils::take_user_input(
        "Password",
        &mut password_string,
        "Enter password to protect account:",
    );

    let mut account_name = String::new();
    utils::take_user_input("Account name", &mut account_name, "Enter account name:");

    account_name = String::from(account_name.trim());

    let account_key = if secret.is_some() {
        Some(secret).unwrap().unwrap()
    } else {
        Mnemonic::<English>::new(&mut rand::thread_rng()).to_phrase()
    };

    account_name.push_str(".json");

    let keystore = web3_keystore::encrypt(
        &mut rand::thread_rng(),
        &account_key,
        password_string.trim(),
        None,
        Some(account_name.clone()),
    )
    .unwrap();

    let account_json = keystore::serialize_keystore(&keystore);

    let mut file_name = String::from("accounts/");
    file_name.push_str(account_name.trim());

    fs::File::create(&file_name).expect("Failed to create file");
    fs::write(&file_name, account_json.as_bytes()).expect("failed to write to file");

    (account_key.clone(), account_name.clone())
}

fn take_secret_input() -> Option<String> {
    let mut user_input = String::new();

    utils::take_user_input(
        "secret",
        &mut user_input,
        "\n\nEnter 12 word seed phrase or private key:",
    );

    let valid_secret = utils::validate_secret_input(&user_input);

    if !valid_secret {
        return None;
    }

    return Some(String::from(user_input.trim()));
}
