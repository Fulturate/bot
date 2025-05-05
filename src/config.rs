use crate::util::currency::converter::{CurrencyConverter, OutputLanguage};
use crate::util::json::{JsonConfig, read_json_config};
use dotenv::dotenv;
use std::sync::Arc;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    #[warn(dead_code)]
    owners: Vec<i64>,
    version: String,
    json_config: JsonConfig,
    currency_converter: Arc<CurrencyConverter>,
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
        let currency_converter = Arc::new(CurrencyConverter::new(OutputLanguage::Russian).unwrap()); // TODO: get language from config

        Config {
            bot,
            owners,
            version,
            json_config,
            currency_converter,
        }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn is_id_in_owners(&self, id: i64) -> bool {
        self.owners.contains(&id)
    }

    pub fn get_json_config(&self) -> &JsonConfig {
        &self.json_config
    }

    pub fn get_currency_converter(&self) -> &CurrencyConverter {
        &self.currency_converter
    }
}
