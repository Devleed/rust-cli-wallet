use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

use crate::provider;

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
}

pub fn get_network_url_by_chain_id(chain_id: &u8) -> &'static str {
    NETWORKS.get(chain_id).unwrap().url.as_str()
}

fn get_chain_ids() -> Vec<&'static u8> {
    NETWORKS.keys().collect()
}

fn get_chain_names() -> Vec<String> {
    NETWORKS
        .values()
        .map(|network| network.name.clone())
        .collect()
}

pub fn change_network_request() {
    let (chain_ids, network_names) = (get_chain_ids(), get_chain_names());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&network_names)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .expect("Failed to create network selection list.");

    let selected_network = chain_ids[selection.unwrap()].clone();

    set_network(selected_network);

    println!("Switched to network: {}", network_names[selection.unwrap()]);
}
