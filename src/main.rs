mod config;
mod handlers;
mod loader;

use crate::loader::run;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    println!("Bot starting...");

    match run().await {
        Ok(_) => println!("Bot stopped"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
