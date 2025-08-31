use crate::db::redis::RedisCache;
use crate::util::currency::converter::{CurrencyConverter, OutputLanguage};
use crate::util::json::{JsonConfig, read_json_config};
use dotenv::dotenv;
use redis::Client as RedisClient;
use std::sync::Arc;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    cobalt_client: ccobalt::Client,
    #[allow(dead_code)]
    owners: Vec<String>,
    log_chat_id: String,
    version: String,
    json_config: JsonConfig,
    currency_converter: Arc<CurrencyConverter>,
    mongodb_url: String,
    redis_client: RedisCache,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let cobalt_api_key = std::env::var("COBALT_API_KEY").expect("COBALT_API_KEY expected");
        let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION expected");
        let bot = Bot::new(bot_token);

        let cobalt_client = ccobalt::Client::builder()
            .base_url("https://cobalt-backend.canine.tools/")
            .api_key(cobalt_api_key)
            .build()
            .expect("Failed to build cobalt client");

        let owners: Vec<String> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        let log_chat_id = std::env::var("LOG_CHAT_ID").expect("LOG_CHAT_ID expected");

        let json_config = read_json_config("config.json").expect("Unable to read config.json");
        let currency_converter = Arc::new(CurrencyConverter::new(OutputLanguage::Russian).unwrap()); // TODO: get language from config
        let mongodb_url = std::env::var("MONGODB_URL").expect("MONGODB_URL expected");

        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL expected");
        let redis_client = RedisClient::open(redis_url).expect("Failed to open Redis client");

        let redis_client = RedisCache::new(redis_client);

        Config {
            bot,
            cobalt_client,
            owners,
            log_chat_id,
            version,
            json_config,
            currency_converter,
            mongodb_url,
            redis_client,
        }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_cobalt_client(&self) -> &ccobalt::Client {
        &self.cobalt_client
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    #[allow(dead_code)]
    pub fn is_id_in_owners(&self, id: String) -> bool {
        self.owners.contains(&id)
    }

    pub fn get_log_chat_id(&self) -> &str {
        &self.log_chat_id
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

    pub fn get_redis_client(&self) -> &RedisCache {
        &self.redis_client
    }
}
