use std::io;

use dialoguer::{console::Term, theme::ColorfulTheme, Select};

const SEED_PHRASE_LEN: usize = 12;
const PKEY_LEN: usize = 64;

pub fn take_user_input(key: &str, msg: &str, validator: Option<fn(&str) -> bool>) -> String {
    let mut input = String::new();

    println!("{}", msg);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to take user input.");

    println!("\n{}: {}", key, input);

    if validator.is_some() {
        while !validator.unwrap()(&input) {
            println!("Invalid {}", key.to_lowercase());
            input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to take user input.");
        }
    }

    return String::from(input.trim());
}
pub fn is_pkey(secret: &str) -> bool {
    !secret.trim().contains(" ")
}
pub fn validate_secret_input(secret: &str) -> bool {
    let is_pkey = is_pkey(secret);

    if is_pkey {
        let pkey = secret.trim().replace("0x", "");

        if pkey.len().ne(&PKEY_LEN) {
            return false;
        }
    } else {
        let count = secret.split_whitespace().count();

        if count.ne(&SEED_PHRASE_LEN) {
            return false;
        }
    }

    return true;
}
pub fn perform_selection(
    key: &str,
    items: &mut Vec<String>,
    heading: Option<&str>,
    with_go_back: bool,
) -> Option<usize> {
    if with_go_back {
        items.push("Go back".to_string());
    }

    if heading.is_some() {
        println!("{}", heading.unwrap());
    }
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap_or_else(|_| {
            println!("failed to create selection list for {}", key);
            return None;
        });

    if with_go_back && selection.unwrap() == items.len() - 1 {
        return None;
    }

    Some(selection.unwrap())
}
pub fn take_valid_password_input(msg: &str) -> String {
    let password_string = take_user_input("Password", msg, Some(|val| val.trim().len().ge(&5)));

    password_string
}
pub fn get_account_path(acc_name: &str) -> String {
    let mut file_name = String::from("accounts/");
    file_name.push_str(acc_name);

    file_name
}
