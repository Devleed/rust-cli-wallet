use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::{fs, vec};
use web3_keystore;

mod keystore;
mod networks;
mod provider;
mod utils;

const SEED_PHRASE_LEN: usize = 12;
const PKEY_LEN: usize = 64;
const CHAIN_ID: u64 = 5;

fn take_secret_input() -> Option<String> {
    let mut user_input = String::new();

    utils::take_user_input(
        "seed phrase",
        &mut user_input,
        "\n\nEnter 12 word seed phrase or private key:",
    );

    let is_pkey = utils::is_pkey(&user_input);

    if is_pkey {
        let pkey = user_input.trim().replace("0x", "");

        if pkey.len().ne(&PKEY_LEN) {
            println!("Invalid private key");

            return None;
        }
    } else {
        let count = user_input.split_whitespace().count();

        if count.ne(&SEED_PHRASE_LEN) {
            println!("Invalid seed phrase");

            return None;
        }
    }

    return Some(String::from(user_input.trim()));
}

async fn send_eth(
    wallet: &Wallet<SigningKey>,
) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let provider = provider::get_provider();

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = wallet.address();

    let balance_from = provider::fetch_balance(address_from).await?;
    // let balance_from: u128 = 10000000000000000000000000000000000;

    let gas_price = provider::fetch_gas_price().await?;
    // let gas_price = 0.00002;

    println!("Available balance: {}", balance_from);

    let mut address_to = String::new();
    utils::take_user_input("Sending to", &mut address_to, "Enter recipient address:");

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
        .to(address_to.trim())
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
        address_to.trim()
    );

    let mut tx_confirmation = String::new();
    utils::take_user_input(
        "confirmation",
        &mut tx_confirmation,
        "Are you sure you want to perform this transaction? [Y/N]",
    );

    if tx_confirmation.trim().to_lowercase() == "y" {
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

fn build_wallet(account_key: &str) -> Wallet<SigningKey> {
    if utils::is_pkey(account_key) {
        account_key
            .parse::<LocalWallet>()
            .expect("Error generating wallet from pkey")
            .with_chain_id(CHAIN_ID)
    } else {
        MnemonicBuilder::<English>::default()
            .phrase(account_key)
            .build()
            .expect("Error generating wallet from seed phrase")
            .with_chain_id(CHAIN_ID)
    }
}

fn create_or_import_wallet(create_new: bool) {
    if !create_new {
        let secret = take_secret_input().unwrap();

        let (account_key, _account_name) = create_new_acc(Some(secret));

        let wallet = build_wallet(&account_key);

        println!("Address: {:?}", wallet.address());
    } else {
        // ? create new acount
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
            let wallet = build_wallet(&account_key);

            println!("Address: {:?}", wallet.address());
        }
    }
}

async fn launch_app() {
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

        // * create file path
        let mut file_name = String::from("accounts/");
        file_name.push_str(selected_value);

        // * read file from given path
        let account_json = fs::read_to_string(file_name.trim()).expect("Failed to read account.");

        let mut password_string = String::new();
        utils::take_user_input("Password", &mut password_string, "Enter password:");

        let secret_key = keystore::deserialize_keystore(&account_json, password_string.trim());

        // * generate wallet from phrase
        let wallet = build_wallet(secret_key.trim());

        println!("Address: {:?}", wallet.address());

        loop {
            launch_authenticated_dashboard(&wallet).await;
        }
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
        send_eth(wallet).await.unwrap();
    } else if selected_action == 1 {
        networks::change_network_request();
    } else if selected_action == 2 {
        let balance = provider::fetch_balance(wallet.address()).await.unwrap();
        println!("balance: {} ETH", balance)
    }
}

#[tokio::main]
async fn main() {
    loop {
        launch_app().await;
    }
}
