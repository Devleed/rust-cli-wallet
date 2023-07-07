mod account;
mod ierc20;
mod keystore;
mod networks;
mod provider;
mod tokens;
mod utils;
mod wallet;

#[tokio::main]
async fn main() {
    loop {
        account::launch_app().await;
    }
}
