mod config;
mod handlers;
mod loader;

use crate::loader::run;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Bot tarting");

    run().await.expect("TODO: panic message");
}
