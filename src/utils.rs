use std::io;

const SEED_PHRASE_LEN: usize = 12;
const PKEY_LEN: usize = 64;

pub fn take_user_input(key: &str, input: &mut String, msg: &str) {
    println!("{}", msg);
    io::stdin()
        .read_line(input)
        .expect("Failed to take user input.");

    println!("\n{}: {}", key, input);
}
pub fn is_pkey(secret: &str) -> bool {
    !secret.trim().contains(" ")
}
pub fn validate_secret_input(secret: &str) -> bool {
    let is_pkey = is_pkey(secret);

    if is_pkey {
        let pkey = secret.trim().replace("0x", "");

        if pkey.len().ne(&PKEY_LEN) {
            println!("Invalid private key");

            return false;
        }
    } else {
        let count = secret.split_whitespace().count();

        if count.ne(&SEED_PHRASE_LEN) {
            println!("Invalid seed phrase");

            return false;
        }
    }

    return true;
}
