use fulturate::bot::dispatcher::run;
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
