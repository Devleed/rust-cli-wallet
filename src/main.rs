use bcrypt;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::signers::coins_bip39::{English, Mnemonic};
use ethers::types::transaction::eip2718::TypedTransaction;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Deserializer;
use serde_json::Serializer;
use std::{fs, io, ops::Add};
use web3_keystore;

const SEED_PHRASE_LEN: usize = 12;
const PKEY_LEN: usize = 64;
const CHAIN_ID: u64 = 5;
const PROVIDER_URL: &str = "https://goerli.infura.io/v3/80ba3747876843469bf0c36d0a355f71";

#[derive(Serialize, Deserialize)]
struct Account {
    phrase: String,
    password: String,
    name: String,
}

fn take_user_input(key: &str, input: &mut String, msg: &str) {
    println!("{}", msg);
    io::stdin()
        .read_line(input)
        .expect("Failed to take user input.");

    println!("\n{}: {}", key, input);
}

fn take_secret_input() -> Option<String> {
    let mut user_input = String::new();

    take_user_input(
        "seed phrase",
        &mut user_input,
        "\n\nEnter 12 word seed phrase or private key:",
    );

    let is_pkey = is_pkey(&user_input);

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

fn get_provider() -> Provider<Http> {
    Provider::<Http>::try_from(PROVIDER_URL).expect("Failed to connect to provider, try again.")
}

async fn fetch_balance(
    address: H160,
    provided_provider: &Provider<Http>,
) -> Result<U256, Box<dyn std::error::Error>> {
    let provider = provided_provider;

    let balance = provider
        .get_balance(address, None)
        .await
        .expect("Failed to fetch user balance");

    println!("balance of {:?} is {} ETH", address, balance);

    Ok(balance)
}

async fn fetch_gas_price(
    provided_provider: &Provider<Http>,
) -> Result<f64, Box<dyn std::error::Error>> {
    let provider = provided_provider;
    let gas_price = provider.get_gas_price().await?;

    Ok(ethers::utils::format_units(gas_price, "ether")?.parse::<f64>()?)
}

async fn send_eth(
    wallet: &Wallet<SigningKey>,
) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let provider = get_provider();

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = wallet.address();

    // let balance_from = fetch_balance(address_from, &provider).await?;
    let balance_from: u128 = 10000000000000000000000000000000000;

    // let gas_price = fetch_gas_price(&provider).await?;
    let gas_price = 0.00002;

    let balance_from = ethers::utils::format_units(balance_from, "ether")?
        .trim()
        .parse::<f64>()?;

    println!("Available balance: {}", balance_from);

    let mut address_to = String::new();
    take_user_input("Sending to", &mut address_to, "Enter recipient address:");

    let mut value_str = String::new();
    take_user_input("value", &mut value_str, "\n\nEnter amount to send in ETH:");

    while value_str.trim().parse::<f64>()?.ge(&balance_from) {
        println!(
            "Amount limit exceeded, sender has {} ETH and you're trying to send {} ETH \n",
            balance_from,
            value_str.trim()
        );
        value_str = String::new();
        take_user_input("value", &mut value_str, "Enter amount to send in ETH:");
    }

    let transaction_req: TypedTransaction = TransactionRequest::new()
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
        address_to
    );

    let mut tx_confirmation = String::new();
    take_user_input(
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

fn serialize_keystore(keystore: &web3_keystore::KeyStore) -> String {
    let mut serializer = Serializer::new(Vec::new());

    keystore.serialize(&mut serializer).unwrap();

    let serialized_data = serializer.into_inner();
    String::from_utf8(serialized_data).unwrap()
}

fn deserialize_keystore(json_string: &str, password: &str) -> String {
    let mut deserializer = Deserializer::from_str(json_string);

    let keystore = web3_keystore::KeyStore::deserialize(&mut deserializer).unwrap();

    let data = web3_keystore::decrypt(&keystore, password).expect("Wrong password");

    String::from_utf8(data).unwrap()
}

fn is_pkey(secret: &str) -> bool {
    !secret.trim().contains(" ")
}

fn create_new_acc(secret: Option<String>) -> (String, String) {
    let mut password_string = String::new();
    take_user_input(
        "Password",
        &mut password_string,
        "Enter password to protect account:",
    );

    let mut account_name = String::new();
    take_user_input("Account name", &mut account_name, "Enter account name:");

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

    let account_json = serialize_keystore(&keystore);

    let mut file_name = String::from("accounts/");
    file_name.push_str(account_name.trim());

    fs::File::create(&file_name).expect("Failed to create file");
    fs::write(&file_name, account_json.as_bytes()).expect("failed to write to file");

    (account_key.clone(), account_name.clone())
}

fn build_wallet(account_key: &str) -> Wallet<SigningKey> {
    if is_pkey(account_key) {
        account_key
            .parse::<LocalWallet>()
            .expect("Error generating wallet from pkey")
    } else {
        MnemonicBuilder::<English>::default()
            .phrase(account_key)
            .build()
            .expect("Error generating wallet from seed phrase")
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
        take_user_input(
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
        take_user_input("Password", &mut password_string, "Enter password:");

        let secret_key = deserialize_keystore(&account_json, password_string.trim());

        // * generate wallet from phrase
        let wallet = build_wallet(secret_key.trim());

        println!("Address: {:?}", wallet.address());

        loop {
            launch_authenticated_dashboard(&wallet).await;
        }
    }
}

async fn launch_authenticated_dashboard(wallet: &Wallet<SigningKey>) {
    let items = vec!["Send eth"];
    let actions = vec![send_eth];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("Failed to create account selection list.");

    let selected_action = actions[selection.unwrap()](wallet);
    let res = selected_action.await.unwrap().unwrap();
}

async fn test() {
    /*
     - create new wallet (build and encrypt mnemonic)
     - import wallet (encrypt mnemonic)
     - import account (encrypt pkey)
     - select account (mnemonic | pkey)
    */

    let mnemonic = Mnemonic::<English>::new_from_phrase(
        "chief width ensure divide height rocket renew vacuum lawsuit link cross plunge",
    )
    .unwrap();

    let wallet = build_wallet(&mnemonic.to_phrase());
    // let serialized_wallet = serde_json::to_string(&wallet);

    // println!("mnemonic: {}", mnemonic.to_phrase());

    // let key = mnemonic
    //     .derive_key("m/44'/60'/0'/0/0", None)
    //     .expect("Failed to derive pkey");

    // let ser_key = serde_json::to_string(&key).unwrap();

    // println!("ser_key ser_key {}", ser_key);

    // let test_key: XPriv = serde_json::from_str(&ser_key).unwrap();

    // println!("key {:?}", test_key);

    // test_key.fingerprint();

    let keystore = web3_keystore::encrypt(
        &mut rand::thread_rng(),
        mnemonic.to_phrase().as_bytes(),
        "karachi12",
        None,
        None,
    )
    .unwrap();

    let mut serializer = Serializer::new(Vec::new());

    keystore.serialize(&mut serializer).unwrap();

    let serialized_data = serializer.into_inner();
    let json_string = String::from_utf8(serialized_data).unwrap();

    println!("Serialized JSON: {}", json_string);

    let mut deserializer = Deserializer::from_str(&json_string);

    let ks = web3_keystore::KeyStore::deserialize(&mut deserializer).unwrap();

    let data = web3_keystore::decrypt(&ks, "karachi12").expect("Wrong password");

    let str_data = String::from_utf8(data).unwrap();

    println!("data {}", str_data)
}

#[tokio::main]
async fn main() {
    // test().await;

    loop {
        launch_app().await;
    }
}
