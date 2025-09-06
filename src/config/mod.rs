use crate::core::db::redis::RedisCache;
use crate::util::currency::converter::{CurrencyConverter, OutputLanguage};
use crate::util::json::{JsonConfig, read_json_config};
use dotenv::dotenv;
use log::error;
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
    error_chat_thread_id: String,
    #[allow(dead_code)]
    warn_chat_thread_id: String,
    version: String,
    json_config: JsonConfig,
    currency_converter: Arc<CurrencyConverter>,
    mongodb_url: String,
    redis_client: RedisCache,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let Ok(bot_token) = std::env::var("BOT_TOKEN") else {
            error!("Expected BOT_TOKEN env var");
            std::process::exit(1);
        };
        let Ok(cobalt_api_key) = std::env::var("COBALT_API_KEY") else {
            error!("COBALT_API_KEY expected");
            std::process::exit(1);
        };
        let Ok(version) = std::env::var("CARGO_PKG_VERSION") else {
            error!("CARGO_PKG_VERSION expected");
            std::process::exit(1);
        };
        let bot = Bot::new(bot_token);

        let cobalt_client = ccobalt::Client::builder()
            .base_url("https://cobalt-backend.canine.tools/")
            .api_key(cobalt_api_key)
            .user_agent(
                "Fulturate/6.6.6 (rust) (+https://github.com/weever1337/fulturate-rs)".to_string(),
            )
            .build()
            .unwrap_or_else(|_err| {
                error!("Failed to build cobalt client");
                std::process::exit(1);
            });

        let owners: Vec<String> = std::env::var("OWNERS")
            .unwrap_or_else(|_| {
                error!("OWNERS expected");
                std::process::exit(1)
            })
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        let Ok(log_chat_id) = std::env::var("LOG_CHAT_ID") else {
            error!("LOG_CHAT_ID expected");
            std::process::exit(1);
        };
        let error_chat_thread_id: String = std::env::var("ERROR_CHAT_THREAD_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.to_string());

        let warn_chat_thread_id: String = std::env::var("WARN_CHAT_THREAD_ID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.to_string());

        let Ok(json_config) = read_json_config("config.json") else {
            error!("Unable to read config.json");
            std::process::exit(1);
        };
        let currency_converter = Arc::new(CurrencyConverter::new(OutputLanguage::Russian).unwrap()); // TODO: get language from config
        let Ok(mongodb_url) = std::env::var("MONGODB_URL") else {
            error!("MONGODB_URL expected");
            std::process::exit(1);
        };

        let Ok(redis_url) = std::env::var("REDIS_URL") else {
            error!("REDIS_URL expected");
            std::process::exit(1);
        };

        let Ok(redis_client) = RedisClient::open(redis_url.to_owned()) else {
            error!("Failed to open Redis client");
            std::process::exit(1);
        };
        let redis_client = RedisCache::new(redis_client);

        Config {
            bot,
            cobalt_client,
            owners,
            log_chat_id,
            error_chat_thread_id,
            warn_chat_thread_id,
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

    pub fn get_error_chat_thread_id(&self) -> &str {
        &self.error_chat_thread_id
    }

    #[allow(dead_code)]
    pub fn get_warn_chat_thread_id(&self) -> &str {
        &self.warn_chat_thread_id
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
