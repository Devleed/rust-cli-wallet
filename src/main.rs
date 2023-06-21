use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::k256::Secp256k1;
use ethers::prelude::*;
use ethers::signers::coins_bip39::{mnemonic, English, Mnemonic};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::fs;
use std::io;

const SEED_PHRASE_LEN: usize = 12;
const CHAIN_ID: u64 = 5;
const PROVIDER_URL: &str = "https://goerli.infura.io/v3/80ba3747876843469bf0c36d0a355f71";
const SENDER_ADDRESS: &str = "0x639268f7E1393347a649B4F371a9DB3065153EE6";
const RECEIVER_ADDRESS: &str = "0xEBa526a6FfB08F081911b2223bdcC59d3374a32A";

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

async fn send_eth(
    wallet: &Wallet<SigningKey>,
) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(PROVIDER_URL)?;

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = SENDER_ADDRESS.parse::<Address>()?;
    let address_to = RECEIVER_ADDRESS.parse::<Address>()?;

    let balance_from = provider.get_balance(address_from, None).await?;
    let balance_to = provider.get_balance(address_to, None).await?;

    let gas_price = provider.get_gas_price().await?;

    let balance_from = ethers::utils::format_units(balance_from, "ether")?;
    let balance_to = ethers::utils::format_units(balance_to, "ether")?;

    println!("{:?} has {} ETH", address_from, balance_from);
    println!("{:?} has {} ETH", address_to, balance_to);

    let mut value_str = String::new();
    take_user_input("value", &mut value_str, "\n\nEnter amount to send in ETH:");

    while value_str
        .trim()
        .parse::<f64>()?
        .ge(&balance_from.trim().parse::<f64>()?)
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
        .to(RECEIVER_ADDRESS)
        .value(U256::from(ethers::utils::parse_ether(value_str.trim())?))
        .into();

    let estimated_gas = provider
        .estimate_gas(&transaction_req, None)
        .await?
        .to_string()
        .parse::<f64>()?;

    let gas_price = ethers::utils::format_units(gas_price, "ether")?.parse::<f64>()?;

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

fn create_new_acc() -> (String, String) {
    let mut account_name = String::new();
    take_user_input("Account name", &mut account_name, "Enter account name:");

    account_name = String::from(account_name.trim());

    let mnemonic = Mnemonic::<English>::new(&mut rand::thread_rng());
    let phrase = mnemonic.to_phrase();

    account_name.push_str(".txt");

    let mut file_name = String::from("accounts/");
    file_name.push_str(account_name.trim());

    fs::File::create(&file_name).expect("Failed to create file");
    fs::write(&file_name, phrase.as_bytes()).expect("failed to write to file");

    (phrase.clone(), account_name.clone())
}

fn build_wallet(mnemonic: &str) -> Wallet<SigningKey> {
    MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .build()
        .expect("Error generating wallet.")
}

#[tokio::main]
async fn main() {
    let accounts = fs::read_dir("accounts").expect("Failed to read directory");

    println!("Available accounts: ");
    for account in accounts.into_iter() {
        println!("name: {}", account.unwrap().file_name().to_str().unwrap());
    }

    let mut account_to_use = String::new();
    take_user_input(
        "Operation",
        &mut account_to_use,
        "Select which account to use or type new to create new account.",
    );

    if account_to_use.trim().to_lowercase() == "new" {
        let (mnemonic, _account_name) = create_new_acc();

        let wallet = build_wallet(&mnemonic);
    } else {
        let mut file_name = String::from("accounts/");
        file_name.push_str(&account_to_use);

        let mnemonic = fs::read_to_string(file_name.trim()).expect("Failed to read account.");

        let wallet = build_wallet(&mnemonic);
    }

    // let wallet = MnemonicBuilder::<English>::default();
    // let result = MnemonicBuilder::write_to(wallet, file_path);

    // println!("{:?}", result);

    // let seed_phrase = take_seed_input().unwrap();

    // let trimmed = seed_phrase.trim();
    // // let trimmed = "silver autumn pet burden energy water wonder river survey observe diary spirit";

    // wallet = wallet.with_chain_id(CHAIN_ID);

    // let address = wallet.address();
    // let chain_id = wallet.chain_id().to_string();

    // println!("Address: {:?}", address);
    // println!("chainID: {} \n", chain_id);

    // let result = send_eth(&wallet).await;

    // match result {
    //     Ok(_res) => println!("Success"),
    //     Err(err) => println!("Error: {}", err.to_string()),
    // }
}
