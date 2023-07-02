use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::provider;

pub const DEFAULT_SELECTED_CHAINID: u8 = 5;

lazy_static! {
    static ref SELECTED_NETWORK: Mutex<u8> = Mutex::new(DEFAULT_SELECTED_CHAINID);
    static ref NETWORKS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            1,
            "https://mainnet.infura.io/v3/80ba3747876843469bf0c36d0a355f71",
        );
        m.insert(
            5,
            "https://goerli.infura.io/v3/80ba3747876843469bf0c36d0a355f71",
        );
        m
    };
    static ref AVAILABLE_NETWORKS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "mainnet");
        m.insert(5, "goerli");
        m
    };
}

pub fn set_network(value: u8) {
    let mut data = SELECTED_NETWORK.lock().unwrap();
    *data = value;

    provider::set_provider(NETWORKS.get(&value).unwrap());
}

pub fn get_network_url() -> String {
    let data = SELECTED_NETWORK.try_lock().expect("Failed to access val");

    NETWORKS.get(&*data).unwrap().to_string()
}

pub fn get_network_url_by_chain_id(chain_id: &u8) -> &str {
    NETWORKS.get(chain_id).unwrap()
}

pub fn get_networks() -> HashMap<u8, &'static str> {
    AVAILABLE_NETWORKS.clone()
}
