use coins_bip32::prelude::SigningKey;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use lazy_static::lazy_static;
use std::fs;
use std::sync::Mutex;

use crate::{keystore, networks, provider, tokens, utils, wallet};

lazy_static! {
    static ref ACCOUNT_KEY: Mutex<Option<String>> = Mutex::new(None);
    static ref ACCOUNT_NAME: Mutex<Option<String>> = Mutex::new(None);
}

pub async fn launch_app() {
    // * read accounts from accounts directory
    let accounts = fs::read_dir("accounts").expect("Failed to read directory");

    // * convert to vector
    let mut account_list = accounts
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<Vec<_>>();

    // * add create new wallet option at the end of list
    account_list.push(String::from("Create new"));
    account_list.push(String::from("Import wallet"));

    // * display list of all accounts for user to select
    println!("Available accounts: ");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&account_list)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("Failed to create account selection list.");

    let selected_value = &account_list[selection.unwrap()].trim().to_lowercase();

    // * check the option selected
    if selected_value == "create new" || selected_value == "import wallet" {
        create_or_import_wallet(selected_value == "create new");
    } else {
        // ? use selected account

        select_wallet(&selected_value);
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

/* PRIVATE FUNCTIONS */

fn create_or_import_wallet(create_new: bool) {
    if !create_new {
        import_wallet();
    } else {
        create_wallet();
    }
}
async fn launch_authenticated_dashboard(wallet: &Wallet<SigningKey>) {
    let items = vec!["Send eth", "Change network", "Display balance", "Add token"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("Failed to create account selection list.");

    let selected_action = selection.unwrap();

    if selected_action == 0 {
        wallet::send_eth().await.unwrap();
    } else if selected_action == 1 {
        networks::change_network_request();
    } else if selected_action == 2 {
        let balance = provider::fetch_balance(wallet.address()).await.unwrap();
        println!("balance: {} ETH", balance)
    } else if selected_action == 3 {
        // * Add token
        tokens::add_token().await;
    }
}
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
fn import_wallet() {
    let secret = take_secret_input().unwrap();

    let (account_key, _account_name) = create_new_acc(Some(secret));

    wallet::build_wallet(&account_key, networks::get_selected_chain_id());
}
fn create_wallet() {
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
        wallet::build_wallet(&account_key, networks::get_selected_chain_id());
    }
}
fn select_wallet(acc_name: &str) {
    // * create file path
    let mut file_name = String::from("accounts/");
    file_name.push_str(acc_name);

    let mut keystore_path = String::from(file_name.clone());
    keystore_path.push_str("/keystore.json");

    // * read file from given path
    let account_json = fs::read_to_string(keystore_path.trim()).expect("Failed to read account.");

    let mut password_string = String::new();
    utils::take_user_input("Password", &mut password_string, "Enter password:");

    let secret_key = keystore::deserialize_keystore(&account_json, password_string.trim());

    let mut data = ACCOUNT_KEY.lock().unwrap();
    *data = Some(secret_key.clone());

    let mut account_name = ACCOUNT_NAME.lock().unwrap();
    *account_name = Some(acc_name.to_string());

    wallet::build_wallet(&secret_key, networks::get_selected_chain_id());
}
