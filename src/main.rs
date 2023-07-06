mod account;
mod keystore;
mod networks;
mod provider;
mod utils;
mod wallet;

#[tokio::main]
async fn main() {
    loop {
        account::launch_app().await;
    }
}
