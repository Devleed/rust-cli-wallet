use std::io;
use ethers::prelude::*;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::signers::coins_bip39::English;
use ethers::types::transaction::eip2718::TypedTransaction;

const SEED_PHRASE_LEN: usize = 12;
const CHAIN_ID: u64 = 5;
const PROVIDER_URL: &str = "https://goerli.infura.io/v3/80ba3747876843469bf0c36d0a355f71";
const SENDER_ADDRESS: &str = "0x639268f7E1393347a649B4F371a9DB3065153EE6";
const RECEIVER_ADDRESS: &str = "0xEBa526a6FfB08F081911b2223bdcC59d3374a32A";

fn take_user_input(key: &str ,input: &mut String, msg: &str) {
    println!("{}", msg);
    io::stdin().read_line(input).expect("Failed to take user input.");

    println!("{key}: {}", input);
}

fn take_seed_input() -> Option<String> {
    let mut user_input = String::new();

    take_user_input("seed phrase", &mut user_input, "Enter 12 word seed phrase:");

    let count = user_input.split_whitespace().count();
    
    if count.ne(&SEED_PHRASE_LEN) {
        println!("Invalid seed phrase");

        return None;
    }

    return Some(String::from(user_input));
}

async fn perform_tx(wallet: &Wallet<SigningKey>) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(PROVIDER_URL)?;

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let address_from = SENDER_ADDRESS.parse::<Address>()?;
    let address_to = RECEIVER_ADDRESS.parse::<Address>()?;

    let balance_from = provider.get_balance(address_from, None).await?;
    let balance_to = provider.get_balance(address_to, None).await?;

    let balance_from = ethers::utils::format_units(balance_from, "ether").expect("error in fetching balance");
    let balance_to = ethers::utils::format_units(balance_to, "ether").expect("error in fetching balance");

    println!("{:?} has {} ETH", address_from, balance_from);
    println!("{:?} has {} ETH", address_to, balance_to);

    let mut value_str = String::new();
    take_user_input("value", &mut value_str, "Enter amount to send:");

    let transaction_req: TypedTransaction = TransactionRequest::new().to(RECEIVER_ADDRESS).value(U256::from(ethers::utils::parse_ether(value_str.trim())?)).into();

    let sent_tx = client.send_transaction(transaction_req, None).await?.await?;

    let receipt = sent_tx.expect("failed to send transaction");

    println!("Tx hash: {:?}", receipt.transaction_hash);

    Ok(receipt)
}

#[tokio::main]
async fn main() {
    let seed_phrase = take_seed_input().unwrap();

    let trimmed = seed_phrase.trim();
    // let trimmed = "silver autumn pet burden energy water wonder river survey observe diary spirit";

    let mut wallet = MnemonicBuilder::<English>::default().phrase(trimmed).build().expect("error creating wallet");

    wallet = wallet.with_chain_id(CHAIN_ID);

    let address = wallet.address();
    let chain_id = wallet.chain_id().to_string();

    println!("Address: {:?}", address);
    println!("chainID: {}", chain_id);

    let result = perform_tx(&wallet).await;
    
    match result {
        Ok(res) => println!("Success"),
        Err(err) => println!("Error: {}", err.to_string())
    }
}

