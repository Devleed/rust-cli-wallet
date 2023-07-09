use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

use crate::{account, provider, utils, wallet};

pub const DEFAULT_SELECTED_CHAINID: u8 = 5;

#[derive(Deserialize)]
struct Network {
    name: String,
    url: String,
}

lazy_static! {
    static ref SELECTED_NETWORK: Mutex<u8> = Mutex::new(DEFAULT_SELECTED_CHAINID);
    static ref NETWORKS: HashMap<u8, Network> = {
        let chains_json =
            fs::read_to_string("config/chains.json").expect("Failed to read account.");
        let network_map: HashMap<u8, Network> = serde_json::from_str(chains_json.trim()).unwrap();

        network_map
    };
}

pub fn set_network(value: u8) {
    let mut data = SELECTED_NETWORK.lock().unwrap();
    *data = value;

    provider::set_provider(NETWORKS.get(&value).unwrap().url.trim());
    wallet::build_wallet(&account::get_account_key().unwrap(), value);
}
pub fn get_network_url_by_chain_id(chain_id: &u8) -> &'static str {
    NETWORKS.get(chain_id).unwrap().url.as_str()
}
pub fn change_network_request() {
    let (chain_ids, network_names) = (get_chain_ids(), get_chain_names());

    let selection =
        utils::perform_selection("Network selection", &network_names, Some("Select network"));

    let selected_network = chain_ids[selection].clone();

    set_network(selected_network);

    println!("Switched to network: {}", network_names[selection]);
}
pub fn get_selected_chain_id() -> u8 {
    let data = SELECTED_NETWORK.lock().unwrap();
    data.clone()
}

/* PRIVATE FUNCTIONS */
fn get_chain_ids() -> Vec<&'static u8> {
    NETWORKS.keys().collect()
}
fn get_chain_names() -> Vec<String> {
    NETWORKS
        .values()
        .map(|network| network.name.clone())
        .collect()
}
