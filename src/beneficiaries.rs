use std::{collections::HashMap, fs};

use ethers::types::{Address, H160};

use crate::{
    account,
    utils::{self, get_account_path, is_valid_ethereum_address, log, LogSeverity},
};

pub fn beneficiary_menu() {
    let mut options = vec![
        "List beneficiaries".to_string(),
        "Add beneficiary".to_string(),
        "Delete beneficiary".to_string(),
    ];

    let selection = utils::perform_selection(
        "Beneficiary",
        &mut options,
        Some("Beneficiary Management"),
        false,
    );

    match selection {
        Some(0) => list_beneficiaries(),
        Some(1) => add_beneficiary(),
        Some(2) => delete_beneficiary(),
        _ => (),
    }
}

pub fn select_beneficiary() -> Option<H160> {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = get_account_path(&account_name);
    account_path.push_str("/beneficiaries.json");

    let beneficiaries_json =
        fs::read_to_string(&account_path).unwrap_or_else(|_e| String::from("{}"));

    let beneficiaries: HashMap<String, Address> =
        serde_json::from_str(beneficiaries_json.trim()).unwrap();

    let mut beneficiary_names: Vec<String> = beneficiaries.keys().map(|key| key.clone()).collect();

    let selection = utils::perform_selection("Beneficiary", &mut beneficiary_names, None, true);

    if selection.is_some() {
        Some(
            beneficiaries
                .get(&beneficiary_names[selection.unwrap()])
                .unwrap()
                .clone(),
        )
    } else {
        None
    }
}

/* PRIVATE FUNCTIONS */

fn add_beneficiary() {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/beneficiaries.json");

    let mut beneficiaries: HashMap<String, Address> = get_beneficiaries_list(&account_path);

    let beneficiary_name = utils::take_user_input(
        "Beneficiary name",
        "Enter beneficiary name to add.\n1. Beneficiary name should be greater than equal to 3.",
        Some(|name| name.trim().len().ge(&3)),
    );

    let beneficiary_address_str = utils::take_user_input(
        "Beneficiary address",
        "Enter beneficiary address to add",
        Some(is_valid_ethereum_address),
    );

    let beneficiary_address: H160 = beneficiary_address_str.trim().parse().unwrap();

    beneficiaries.insert(String::from(beneficiary_name.trim()), beneficiary_address);

    let new_beneficiaries_str = serde_json::to_string(&beneficiaries).unwrap();
    fs::write(&account_path, new_beneficiaries_str.as_bytes()).unwrap();

    log("Beneficiary added successfully", Some(LogSeverity::INFO));
}

fn delete_beneficiary() {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/beneficiaries.json");

    let mut beneficiaries: HashMap<String, Address> = get_beneficiaries_list(&account_path);

    let beneficiary_name = utils::take_user_input(
        "Beneficiary name",
        "Enter beneficiary name to delete.\n",
        Some(|name| name.trim().len().ge(&3)),
    );

    beneficiaries.remove(&beneficiary_name);

    let new_beneficiaries_str = serde_json::to_string(&beneficiaries).unwrap();
    fs::write(&account_path, new_beneficiaries_str.as_bytes()).unwrap();

    log("Beneficiary deleted successfully", Some(LogSeverity::INFO));
}

fn list_beneficiaries() {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/beneficiaries.json");

    let beneficiaries: HashMap<String, Address> = get_beneficiaries_list(&account_path);

    if beneficiaries.len() == 0 {
        log("No beneficiaries found", Some(LogSeverity::INFO));
    } else {
        log("List of beneficiaries", Some(LogSeverity::INFO));

        for (name, address) in beneficiaries.iter() {
            println!("{}: {}", name, address);
        }
    }
}

fn get_beneficiaries_list(account_path: &str) -> HashMap<String, Address> {
    let beneficiaries_json = fs::read_to_string(&account_path).unwrap_or_else(|_e| {
        fs::write(&account_path, "{}".as_bytes()).unwrap();

        fs::read_to_string(&account_path).unwrap()
    });

    let beneficiaries: HashMap<String, Address> =
        serde_json::from_str(beneficiaries_json.trim()).unwrap();

    return beneficiaries;
}
