use coins_bip32::prelude::SigningKey;
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use lazy_static::lazy_static;
use std::io::prelude::*;
use std::ops::Mul;
use std::sync::Mutex;
use std::{fs, panic};

use crate::fiat;
use crate::networks::get_selected_chain_coin;
use crate::provider::estimate_gas;
use crate::utils::{get_account_path, log, LogSeverity};
use crate::wallet::get_wallet;
use crate::{beneficiaries, keystore, networks, provider, tokens, utils, wallet};

lazy_static! {
    static ref ACCOUNT_KEY: Mutex<Option<String>> = Mutex::new(None);
    static ref ACCOUNT_NAME: Mutex<Option<String>> = Mutex::new(None);
}

pub async fn launch_app() {
    // * read accounts from accounts directory
    let accounts = match fs::read_dir("accounts") {
        Ok(entries) => Some(entries),
        Err(err) => {
            eprintln!("Error reading accounts directory: {}", err);
            None
        }
    };

    // * convert to vector
    let mut account_list = if accounts.is_none() {
        Vec::new()
    } else {
        accounts
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().into_string().unwrap())
            .collect::<Vec<_>>()
    };

    // * add create new wallet option at the end of list
    account_list.push(String::from("Create new"));
    account_list.push(String::from("Import wallet"));

    // * display list of all accounts for user to select
    let selection = utils::perform_selection(
        "Accounts",
        &mut account_list,
        Some("Available accounts"),
        false,
    );

    let selected_value = &account_list[selection.unwrap()].trim();

    // * check the option selected
    if *selected_value == "Create new" || *selected_value == "Import wallet" {
        create_or_import_wallet(*selected_value == "Create new");
    } else {
        // ? use selected account

        select_account(&selected_value);
        let wallet = wallet::get_wallet();

        if wallet.is_some() {
            let wallet = wallet.unwrap();
            let connected_network = networks::get_selected_chain_name();

            log("Logged in successfully", Some(LogSeverity::INFO));
            println!("wallet address: {:?}", wallet.address());
            println!("connected network: {}", connected_network);

            fiat::set_fiat_rate("ETH").await;

            loop {
                let res = launch_authenticated_dashboard(&wallet).await;
                if res {
                    break;
                }
            }
        }
    }
}
pub fn get_account_key() -> Option<String> {
    let data = ACCOUNT_KEY.lock().expect("Failed to lock acc key");

    data.clone()
}
pub fn set_account_key(key: Option<String>) {
    let mut data = ACCOUNT_KEY.lock().unwrap();
    *data = key;
}
pub fn get_account_name() -> Option<String> {
    let data = ACCOUNT_NAME.lock().expect("Failed to lock acc name");

    data.clone()
}
pub fn set_account_name(name: Option<String>) {
    let mut data = ACCOUNT_NAME.lock().unwrap();
    *data = name;
}
pub async fn launch_token_actions(token: &tokens::Token) {
    loop {
        let mut actions = vec!["Display balance".to_string(), "Send".to_string()];
        let wallet = get_wallet().unwrap();

        let selection = utils::perform_selection("Token actions", &mut actions, None, true);

        if selection.is_none() {
            break;
        } else if selection.unwrap() == 0 {
            let balance = tokens::fetch_token_balance(token.address, wallet.address()).await;

            println!("User balance: {} {}", balance, token.name);
        } else if selection.unwrap() == 1 {
            tokens::send_token(token).await;
        }
    }
}

/* PRIVATE FUNCTIONS */

fn create_or_import_wallet(create_new: bool) {
    if !create_new {
        import_wallet();
    } else {
        create_wallet();
    }
}
async fn launch_authenticated_dashboard(wallet: &Wallet<SigningKey>) -> bool {
    println!("=======================================================================");
    let mut items = vec![
        "Send native token".to_string(),
        "Change network".to_string(),
        "Display balance".to_string(),
        "Add token".to_string(),
        "Select token".to_string(),
        "Beneficiary management".to_string(),
        "Change password".to_string(),
        "Check gas prices".to_string(),
        "Delete account (danger)".to_string(),
    ];

    let selection = utils::perform_selection("Authenticated dashboard", &mut items, None, false);

    let selected_action = selection.unwrap();

    if selected_action == 0 {
        // send eth flow
        wallet::send_eth().await.unwrap();
        return false;
    } else if selected_action == 1 {
        // change network flow
        networks::change_network_request().await;
        return false;
    } else if selected_action == 2 {
        // display user balance
        let balance = provider::fetch_balance(wallet.address()).await.unwrap();
        let coin = networks::get_selected_chain_coin();
        let fiat_rate = fiat::get_fiat_rate();
        println!(
            "balance: {} {} ({} USD)",
            balance,
            coin,
            balance.mul(fiat_rate)
        );
        return false;
    } else if selected_action == 3 {
        // add token flow
        tokens::add_token().await;
        return false;
    } else if selected_action == 4 {
        // select token flow
        let tokens: Vec<tokens::Token> = tokens::get_user_tokens();
        let mut token_names: Vec<String> =
            tokens.clone().into_iter().map(|token| token.name).collect();

        let selection =
            utils::perform_selection("Tokens", &mut token_names, Some("Select token:"), true);

        if selection.is_some() {
            let selected_token = &tokens[selection.unwrap()];
            launch_token_actions(selected_token).await;
        }
        return false;
    } else if selected_action == 5 {
        // add beneficiary flow
        beneficiaries::beneficiary_menu();
        return false;
    } else if selected_action == 6 {
        // change password flow
        change_password();
        return false;
    } else if selected_action == 7 {
        if let Some(mut dummy_tx) = wallet::create_dummy_send_tx().await {
            let estimated_gas = estimate_gas(&mut dummy_tx, None).await;
            let selected_coin = get_selected_chain_coin();
            let fiat_rate = fiat::get_fiat_rate();

            println!(
                "Current gas price is estimated to be: {} {} ({} USD)",
                estimated_gas,
                selected_coin,
                estimated_gas.mul(fiat_rate)
            );
        }

        return false;
    } else if selected_action == 8 {
        delete_account();
        return true;
    }

    return false;
}
fn create_new_acc(secret: Option<String>) -> (String, String) {
    let password_string = utils::take_valid_password_input(
        "Enter password to protect account: \n1. Password should be of atleast 6 characters.",
    );

    let account_name = utils::take_user_input(
        "Account name",
        "Enter account name:\n1. Account name should be of atleast 3 characters.\n2. If the name of account already exists it will replace the old account with this one.",
        Some(|val| val.trim().len().gt(&3)),
    );

    let account_key = if secret.is_some() {
        secret.unwrap()
    } else {
        let phrase = Mnemonic::<English>::new(&mut rand::thread_rng()).to_phrase();

        println!("{}", phrase);

        let mut confirmation = utils::take_user_input(
            "confirmation",
            "Have you saved this seed phrase somewhere? [y/n]",
            None,
        );

        while confirmation.trim().to_lowercase() != "y" {
            log(
                "Please save this seed phrase somewhere",
                Some(LogSeverity::INFO),
            );
            confirmation = utils::take_user_input(
                "confirmation",
                "Have you saved this seed phrase somewhere? [y/n]",
                None,
            );
        }

        phrase
    };

    let keystore = web3_keystore::encrypt(
        &mut rand::thread_rng(),
        &account_key,
        password_string.trim(),
        None,
        Some(account_name.clone()),
    )
    .unwrap();

    let account_json = keystore::serialize_keystore(&keystore);

    create_account_file(&account_name, &account_json);

    (account_key.clone(), account_name.clone())
}
fn take_secret_input() -> String {
    let user_input = utils::take_user_input(
        "secret",
        "\n\nEnter 12 word seed phrase or private key:",
        Some(|val| utils::validate_secret_input(val)),
    );

    return String::from(user_input.trim());
}
fn import_wallet() {
    let secret = take_secret_input();

    let (account_key, _account_name) = create_new_acc(Some(secret));

    wallet::build_wallet(&account_key, networks::get_selected_chain_id());
}
fn create_wallet() {
    let create_new_acc_confirmation = utils::take_user_input(
        "Confirmation",
        "Do you want to create a new wallet? [Y/N]",
        None,
    );

    if create_new_acc_confirmation.trim().to_lowercase() == "y" {
        // * create new wallet
        let (account_key, _account_name) = create_new_acc(None);

        // * generate wallet from phrase
        wallet::build_wallet(&account_key, networks::get_selected_chain_id());
    }
}
fn try_deserializing_account(
    acc_name: &str,
    password: &str,
) -> Result<String, web3_keystore::KeyStoreError> {
    // * create file path
    let file_name = get_account_path(acc_name);
    println!("path name {}", file_name);

    let mut keystore_path = String::from(&file_name);
    keystore_path.push_str("/keystore.json");

    // * read file from given path
    let account_json = fs::read_to_string(keystore_path.trim()).expect("Failed to read account.");

    keystore::deserialize_keystore(&account_json, password)
}
fn select_account(acc_name: &str) {
    let password_string = utils::take_valid_password_input("Enter password:");

    let secret_key = try_deserializing_account(acc_name, &password_string);

    if secret_key.is_ok() {
        set_account_key(Some(secret_key.as_ref().unwrap().clone()));
        set_account_name(Some(acc_name.to_string()));

        wallet::build_wallet(&secret_key.unwrap(), networks::get_selected_chain_id());
    } else {
        let error = secret_key.err();

        if keystore::is_wrong_password(error.unwrap()) {
            log(
                "Wrong password, please try again.",
                Some(LogSeverity::ERROR),
            );
            select_account(acc_name)
        } else {
            panic!("Something went wrong.");
        }
    }
}
fn create_account_file(account_name: &str, account_json: &str) {
    let mut folder_name = String::from("accounts/");
    folder_name.push_str(account_name.trim());

    if fs::metadata(&folder_name).is_err() {
        fs::create_dir(&folder_name).expect("Failed to create user account folder");
    }

    let mut file_path = String::from(folder_name);
    file_path.push_str("/keystore.json");

    let mut created_file =
        fs::File::create(file_path).expect("Failed to create user account file.");

    created_file
        .write_all(account_json.as_bytes())
        .expect("failed to write to file");
}
fn change_password() {
    let acc_name = get_account_name().unwrap();

    let secret_key = authenticate_account(&acc_name);

    let new_password = utils::take_valid_password_input(
        "Enter new password: \n1. Password should be of atleast 6 characters.",
    );

    let keystore = web3_keystore::encrypt(
        &mut rand::thread_rng(),
        &secret_key,
        new_password.trim(),
        None,
        Some(acc_name.clone()),
    )
    .unwrap();

    let account_json = keystore::serialize_keystore(&keystore);

    create_account_file(&acc_name, &account_json);

    log("Password updated successfully.", Some(LogSeverity::INFO));
}
fn delete_account() {
    let acc_name = get_account_name().unwrap();

    authenticate_account(&acc_name);

    let account_path = get_account_path(&acc_name);

    fs::remove_dir_all(&account_path).expect("Failed to delete account.");

    set_account_key(None);
    set_account_name(None);
    wallet::set_wallet(None);
}
fn authenticate_account(acc_name: &str) -> String {
    let old_password = utils::take_valid_password_input("Enter password");

    let result = try_deserializing_account(acc_name, &old_password);

    if result.is_err() {
        log(
            "Wrong password, please try again.",
            Some(LogSeverity::ERROR),
        );
        return authenticate_account(acc_name);
    }

    result.unwrap().clone()
}
