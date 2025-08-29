mod config;
pub(crate) mod db;
mod handlers;
mod loader;
mod util;

use crate::loader::run;
use log::{error, info};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    info!("Bot starting...");

    match run().await {
        Ok(_) => info!("Bot stopped"),
        Err(e) => error!("Bot run failed: {}", e),
    }
}
