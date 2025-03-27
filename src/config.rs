use dotenv::dotenv;
use teloxide::prelude::*;
use crate::util::json::{read_json_config, JsonConfig};

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    owners: Vec<i64>,
    version: String,
    json_config: JsonConfig
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION expected");
        let bot = Bot::new(bot_token);

        let owners: Vec<i64> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        let json_config = read_json_config("config.json").expect("Unable to read config.json");

        Config { bot, owners, version, json_config}
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn get_owners(&self) -> &Vec<i64> {
        &self.owners
    }

    pub fn get_json_config(&self) -> &JsonConfig {
        &self.json_config
    }
}
