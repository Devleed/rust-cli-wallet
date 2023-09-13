use std::{collections::HashMap, fs};

use ethers::types::{Address, H160};

use crate::{account, utils};

pub fn add_beneficiary() {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
    account_path.push_str("/beneficiaries.json");

    let beneficiaries_json = fs::read_to_string(&account_path).unwrap_or_else(|_e| {
        fs::write(&account_path, "{}".as_bytes()).unwrap();

        fs::read_to_string(&account_path).unwrap()
    });

    let mut beneficiaries: HashMap<String, Address> =
        serde_json::from_str(beneficiaries_json.trim()).unwrap();

    let beneficiary_name = utils::take_user_input("Beneficiary name", "Enter beneficiary name");

    let beneficiary_address_str =
        utils::take_user_input("Beneficiary address", "Enter beneficiary address");

    let beneficiary_address: H160 = beneficiary_address_str.trim().parse().unwrap();

    beneficiaries.insert(String::from(beneficiary_name.trim()), beneficiary_address);

    let new_beneficiaries_str = serde_json::to_string(&beneficiaries).unwrap();
    fs::write(&account_path, new_beneficiaries_str.as_bytes()).unwrap();

    println!("Beneficiary added successfully");
}

pub fn select_beneficiary() -> Option<H160> {
    let account_name = account::get_account_name().unwrap();

    let mut account_path = String::from("accounts/");
    account_path.push_str(&account_name);
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
