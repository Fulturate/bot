use std::sync::Arc;
use dotenv::dotenv;
use google_generative_ai_rs::v1::api::Client;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    owners: Vec<i64>,
    ai_client: Arc<Client>,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let ai_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY expected");
        let bot = Bot::new(bot_token);
        let ai_client = Client::new(ai_key);

        let owners: Vec<i64> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        Config {
            bot,
            owners,
            ai_client: Arc::new(ai_client),
        }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_owners(&self) -> &Vec<i64> {
        &self.owners
    }

    pub fn get_ai_client(&self) -> &Client {
        &self.ai_client
    }
}
