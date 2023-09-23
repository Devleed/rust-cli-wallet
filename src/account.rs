use coins_bip32::prelude::SigningKey;
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use lazy_static::lazy_static;
use std::fs;
use std::sync::Mutex;

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
    if *selected_value == "create new" || *selected_value == "import wallet" {
        create_or_import_wallet(*selected_value == "create new");
    } else {
        // ? use selected account

        select_account(&selected_value);
        let wallet = wallet::get_wallet();

        if wallet.is_some() {
            let wallet = wallet.unwrap();

            println!("wallet: {:?}", wallet.address());

            loop {
                launch_authenticated_dashboard(&wallet).await;
            }
        }
    }
}
pub fn get_account_key() -> Option<String> {
    let data = ACCOUNT_KEY.lock().expect("Failed to lock acc key");

    data.clone()
}
pub fn get_account_name() -> Option<String> {
    let data = ACCOUNT_NAME.lock().expect("Failed to lock acc name");

    data.clone()
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
async fn launch_authenticated_dashboard(wallet: &Wallet<SigningKey>) {
    let mut items = vec![
        "Send eth".to_string(),
        "Change network".to_string(),
        "Display balance".to_string(),
        "Add token".to_string(),
        "Select token".to_string(),
        "Add beneficiary".to_string(),
        "Change password".to_string(),
    ];

    let selection = utils::perform_selection("Authenticated dashboard", &mut items, None, false);

    let selected_action = selection.unwrap();

    if selected_action == 0 {
        // send eth flow
        wallet::send_eth().await.unwrap();
    } else if selected_action == 1 {
        // change network flow
        networks::change_network_request();
    } else if selected_action == 2 {
        // display user balance
        let balance = provider::fetch_balance(wallet.address()).await.unwrap();
        println!("balance: {} ETH", balance)
    } else if selected_action == 3 {
        // add token flow
        tokens::add_token().await;
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
    } else if selected_action == 5 {
        // add beneficiary flow
        beneficiaries::add_beneficiary();
    } else if selected_action == 6 {
        // change password flow
    }
}
fn create_new_acc(secret: Option<String>) -> (String, String) {
    let mut password_string =
        utils::take_user_input("Password", "Enter password to protect account:");

    while password_string.trim().len() < 5 {
        println!("Password should be atleast of 6 characters");
        password_string = utils::take_user_input("Password", "Enter password to protect account:");
    }

    let mut account_name = utils::take_user_input("Account name", "Enter account name:");

    while account_name.trim().len() < 3 {
        account_name = utils::take_user_input("Account name", "Enter account name:");
    }

    account_name = String::from(account_name.trim());

    let account_key = if secret.is_some() {
        secret.unwrap()
    } else {
        let phrase = Mnemonic::<English>::new(&mut rand::thread_rng()).to_phrase();

        println!("{}", phrase);

        let mut confirmation = utils::take_user_input(
            "confirmation",
            "Have you saved this seed phrase somewhere? [y/n]",
        );

        while confirmation.trim() != "y" {
            println!("Please save this seed phrase somewhere");
            confirmation = utils::take_user_input(
                "confirmation",
                "Have you saved this seed phrase somewhere? [y/n]",
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

    let mut folder_name = String::from("accounts/");
    folder_name.push_str(account_name.trim());

    fs::create_dir(&folder_name).expect("Failed to create user account folder");

    let mut file_name = String::from(folder_name);
    file_name.push_str("/keystore.json");

    fs::File::create(&file_name).expect("Failed to create file");
    fs::write(&file_name, account_json.as_bytes()).expect("failed to write to file");

    (account_key.clone(), account_name.clone())
}
fn take_secret_input() -> Option<String> {
    let user_input =
        utils::take_user_input("secret", "\n\nEnter 12 word seed phrase or private key:");

    let valid_secret = utils::validate_secret_input(&user_input);

    if !valid_secret {
        return None;
    }

    return Some(String::from(user_input.trim()));
}
fn import_wallet() {
    let mut secret = take_secret_input();

    while secret.is_none() {
        println!("Invalid seed phrase or private key.");
        secret = take_secret_input();
    }

    let (account_key, _account_name) = create_new_acc(secret);

    wallet::build_wallet(&account_key, networks::get_selected_chain_id());
}
fn create_wallet() {
    let create_new_acc_confirmation =
        utils::take_user_input("Confirmation", "Do you want to create a new wallet? [Y/N]");

    if create_new_acc_confirmation.trim().to_lowercase() == "y" {
        // * create new wallet
        let (account_key, _account_name) = create_new_acc(None);

        // * generate wallet from phrase
        wallet::build_wallet(&account_key, networks::get_selected_chain_id());
    }
}
fn select_account(acc_name: &str) {
    // * create file path
    let mut file_name = String::from("accounts/");
    file_name.push_str(acc_name);

    println!("path name {}", file_name);

    let mut keystore_path = String::from(&file_name);
    keystore_path.push_str("/keystore.json");

    // * read file from given path
    let account_json = fs::read_to_string(keystore_path.trim()).expect("Failed to read account.");

    let password_string = utils::take_user_input("Password", "Enter password:");

    let secret_key = keystore::deserialize_keystore(&account_json, password_string.trim());

    let mut data = ACCOUNT_KEY.lock().unwrap();
    *data = Some(secret_key.clone());

    let mut account_name = ACCOUNT_NAME.lock().unwrap();
    *account_name = Some(acc_name.to_string());

    wallet::build_wallet(&secret_key, networks::get_selected_chain_id());
}
fn change_password() {}
