use crate::util::currency::converter::{CurrencyConverter, OutputLanguage};
use crate::util::json::{JsonConfig, read_json_config};
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    #[allow(dead_code)]
    owners: Vec<String>,
    version: String,
    json_config: JsonConfig,
    currency_converter: Arc<CurrencyConverter>,
    mongodb_url: String,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION expected");
        let bot = Bot::new(bot_token);

        let owners: Vec<String> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        let json_config = read_json_config("config.json").expect("Unable to read config.json");
        let currency_converter = Arc::new(CurrencyConverter::new(OutputLanguage::Russian).unwrap()); // TODO: get language from config
        let mongodb_url = std::env::var("MONGODB_URL").expect("MONGODB_URL expected");

        Config {
            bot,
            owners,
            version,
            json_config,
            currency_converter,
            mongodb_url,
        }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    #[allow(dead_code)]
    pub fn is_id_in_owners(&self, id: String) -> bool {
        self.owners.contains(&id)
    }

    pub fn get_json_config(&self) -> &JsonConfig {
        &self.json_config
    }

    pub fn get_currency_converter(&self) -> &CurrencyConverter {
        &self.currency_converter
    }

    pub fn get_mongodb_url(&self) -> &str {
        &self.mongodb_url
    }
}
