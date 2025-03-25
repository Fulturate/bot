use dotenv::dotenv;
use gemini_client_rs::GeminiClient;
use std::sync::Arc;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    owners: Vec<i64>,
    client: Arc<GeminiClient>,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let ai_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY expected");
        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let bot = Bot::new(bot_token);

        let owners: Vec<i64> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        Config {
            bot,
            owners,
            client: Arc::new(GeminiClient::new(ai_key)),
        }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_owners(&self) -> &Vec<i64> {
        &self.owners
    }

    pub fn get_client(&self) -> Arc<GeminiClient> {
        self.client.clone()
    }
}
