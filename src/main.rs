use bcrypt;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::signers::coins_bip39::{English, Mnemonic};
use ethers::types::transaction::eip2718::TypedTransaction;
// use serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Deserializer;
use serde_json::Serializer;
use std::{fs, io, ops::Add};
use web3_keystore;

const SEED_PHRASE_LEN: usize = 12;
const CHAIN_ID: u64 = 5;
const PROVIDER_URL: &str = "https://goerli.infura.io/v3/80ba3747876843469bf0c36d0a355f71";
const SENDER_ADDRESS: &str = "0x639268f7E1393347a649B4F371a9DB3065153EE6";
const RECEIVER_ADDRESS: &str = "0xEBa526a6FfB08F081911b2223bdcC59d3374a32A";
const HASH_COST: u32 = 5;

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

fn take_seed_input() -> Option<String> {
    let mut user_input = String::new();

    take_user_input(
        "seed phrase",
        &mut user_input,
        "\n\nEnter 12 word seed phrase:",
    );

    let count = user_input.split_whitespace().count();

    if count.ne(&SEED_PHRASE_LEN) {
        println!("Invalid seed phrase");

        return None;
    }

    return Some(String::from(user_input));
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
    let mut address_to = String::new();
    take_user_input("Sending to", &mut address_to, "Enter recipient address:");

    let balance_from = fetch_balance(address_from, &provider).await?;

    let gas_price = fetch_gas_price(&provider).await?;

    let balance_from = ethers::utils::format_units(balance_from, "ether")?
        .trim()
        .parse::<f64>()?;

    println!("Available balance: {}", balance_from);

    let mut value_str = String::new();
    take_user_input("value", &mut value_str, "\n\nEnter amount to send in ETH:");

    while value_str
        .trim()
        .parse::<f64>()?
        .add(gas_price)
        .ge(&balance_from)
    {
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

fn create_new_acc(seed: Option<String>) -> (String, String) {
    let password = set_password();

    let mut account_name = String::new();
    take_user_input("Account name", &mut account_name, "Enter account name:");

    account_name = String::from(account_name.trim());

    let mnemonic = if seed.is_some() {
        Mnemonic::<English>::new_from_phrase(Some(seed).unwrap().unwrap().trim()).unwrap()
    } else {
        Mnemonic::<English>::new(&mut rand::thread_rng())
    };
    let phrase = mnemonic.to_phrase();

    account_name.push_str(".json");

    let new_account = Account {
        name: account_name.clone().replace(".json", ""),
        phrase: phrase.clone(),
        password,
    };

    let account_json = serde_json::to_string(&new_account).expect("Failed to generate json");

    let mut file_name = String::from("accounts/");
    file_name.push_str(account_name.trim());

    fs::File::create(&file_name).expect("Failed to create file");
    fs::write(&file_name, account_json.as_bytes()).expect("failed to write to file");

    (phrase.clone(), account_name.clone())
}

fn build_wallet(mnemonic: &str) -> Wallet<SigningKey> {
    MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .build()
        .expect("Error generating wallet.")
}

fn set_password() -> String {
    let mut password_string = String::new();
    take_user_input(
        "Password",
        &mut password_string,
        "Enter password to protect account:",
    );

    bcrypt::hash(&password_string, HASH_COST).unwrap()
}

fn verify_password(hash: &str) -> bool {
    let mut password_string = String::new();
    take_user_input("Password", &mut password_string, "Enter password:");

    bcrypt::verify(password_string, hash).unwrap()
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
    account_list.push(String::from("Import account"));

    // * display list of all accounts for user to select
    println!("Available accounts: ");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&account_list)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("Failed to create account selection list.");

    let selected_value = &account_list[selection.unwrap()].trim().to_lowercase();

    // * check the option selected
    if selected_value == "create new" || selected_value == "import account" {
        if selected_value == "import account" {
            let seed_phrase = take_seed_input().unwrap();

            let (mnemonic, _account_name) = create_new_acc(Some(seed_phrase));

            let wallet = build_wallet(&mnemonic);

            println!("Address: {:?}", wallet.address());
        }

        // ? create new acount
        let mut create_new_acc_confirmation = String::new();
        take_user_input(
            "Confirmation",
            &mut create_new_acc_confirmation,
            "Do you want to create a new account? [Y/N]",
        );

        if create_new_acc_confirmation.trim().to_lowercase() == "y" {
            // * create new wallet
            let (mnemonic, _account_name) = create_new_acc(None);

            // * generate wallet from phrase
            let wallet = build_wallet(&mnemonic);

            println!("Address: {:?}", wallet.address());
        }
    } else {
        // ? use selected account

        // * create file path
        let mut file_name = String::from("accounts/");
        file_name.push_str(selected_value);

        // * read file from given path
        let account_json = fs::read_to_string(file_name.trim()).expect("Failed to read account.");

        // * deserialize json to account
        let account: Account =
            serde_json::from_str(account_json.as_str()).expect("Failed to deserialize.");

        let verified = verify_password(&account.password);

        if verified {
            // * generate wallet from phrase
            let wallet = build_wallet(&account.phrase);

            println!("Address: {:?}", wallet.address());

            loop {
                launch_authenticated_dashboard(&wallet).await;
            }
        } else {
            println!("Incorrect password");
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
    let mnemonic = Mnemonic::<English>::new_from_phrase(
        "chief width ensure divide height rocket renew vacuum lawsuit link cross plunge",
    )
    .unwrap();

    println!("mnemonic: {}", mnemonic.to_phrase());

    let key = mnemonic
        .derive_key("m/44'/60'/0'/0/0", None)
        .expect("Failed to derive pkey");

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

    let data = web3_keystore::decrypt(&ks, "karachi123").expect("Wrong password");

    let str_data = String::from_utf8(data).unwrap();

    println!("data {}", str_data)
}

#[tokio::main]
async fn main() {
    test().await;

    // loop {
    //     launch_app().await;
    // }
}
