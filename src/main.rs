use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::{fs, vec};

mod keystore;
mod networks;
mod provider;
mod utils;
mod wallet;

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

fn create_or_import_wallet(create_new: bool) {
    if !create_new {
        wallet::import_wallet();
    } else {
        wallet::create_wallet();
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
