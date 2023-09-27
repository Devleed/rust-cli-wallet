use coins_bip32::prelude::SigningKey;
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use lazy_static::lazy_static;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;
use std::{fs, panic};

use crate::utils::get_account_path;
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
        change_password();
    }
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

        while confirmation.trim() != "y" {
            println!("Please save this seed phrase somewhere");
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
    let password_string = utils::take_user_input("Password", "Enter password:", None);

    let secret_key = try_deserializing_account(acc_name, &password_string);

    if secret_key.is_ok() {
        let mut data = ACCOUNT_KEY.lock().unwrap();
        *data = Some(secret_key.as_ref().unwrap().clone());

        let mut account_name = ACCOUNT_NAME.lock().unwrap();
        *account_name = Some(acc_name.to_string());

        wallet::build_wallet(&secret_key.unwrap(), networks::get_selected_chain_id());
    } else {
        let error = secret_key.err();

        if keystore::is_wrong_password(error.unwrap()) {
            println!("Wrong password, please try again.");
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

    // if fs::metadata(&file_path).is_ok() {
    //     fs::remove_file(&file_path).expect("Failed to remove the existing account file.");
    // }

    // Create and open the file with write mode
    let mut created_file =
        fs::File::create(file_path).expect("Failed to create user account file.");

    created_file
        .write_all(account_json.as_bytes())
        .expect("failed to write to file");
}
fn change_password() {
    let old_password = utils::take_valid_password_input("Enter old password");
    let acc_name = get_account_name().unwrap();

    let result = try_deserializing_account(&acc_name, &old_password);

    if result.is_err() {
        println!("Wrong password, try again.");
        change_password();
    }

    println!("Unwrapping");
    let secret_key = result.unwrap();

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

    println!("Password updated successfully.");
}
