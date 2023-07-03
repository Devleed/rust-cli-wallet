use std::sync::Mutex;

use coins_bip32::prelude::SigningKey;
use ethers::signers::Wallet;
use lazy_static::lazy_static;

lazy_static! {
    static ref WALLET: Mutex<Option<Wallet<SigningKey>>> = Mutex::new(None);
    static ref ACCOUNT_KEY: Mutex<Option<&'static str>> = Mutex::new(None);
}

pub fn select_wallet(acc_name: &str) {}
