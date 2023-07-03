use std::io;

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
