use coins_bip32::prelude::SigningKey;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use std::fs;

use crate::{keystore, networks, provider, utils, wallet};

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

        wallet::select_wallet(&selected_value);
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
pub fn create_new_acc(secret: Option<String>) -> (String, String) {
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
pub fn take_secret_input() -> Option<String> {
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

/* PRIVATE FUNCTIONS */

fn create_or_import_wallet(create_new: bool) {
    if !create_new {
        wallet::import_wallet();
    } else {
        wallet::create_wallet();
    }
}
async fn launch_authenticated_dashboard(wallet: &Wallet<SigningKey>) {
    let items = vec!["Send eth", "Change network", "Display balance"];

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
    }
}
