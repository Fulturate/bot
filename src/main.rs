mod config;
mod handlers;
mod loader;
mod util;
pub(crate) mod db;

use crate::loader::run;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("Bot starting...");

    match run().await {
        Ok(_) => println!("Bot stopped"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
