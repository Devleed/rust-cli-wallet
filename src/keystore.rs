use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Deserializer;
use serde_json::Serializer;
use web3_keystore;

pub fn serialize_keystore(keystore: &web3_keystore::KeyStore) -> String {
    let mut serializer = Serializer::new(Vec::new());

    keystore.serialize(&mut serializer).unwrap();

    let serialized_data = serializer.into_inner();
    String::from_utf8(serialized_data).unwrap()
}
pub fn deserialize_keystore(json_string: &str, password: &str) -> String {
    let mut deserializer = Deserializer::from_str(json_string);

    let keystore = web3_keystore::KeyStore::deserialize(&mut deserializer).unwrap();

    let data = web3_keystore::decrypt(&keystore, password).expect("Wrong password");

    String::from_utf8(data).unwrap()
}
