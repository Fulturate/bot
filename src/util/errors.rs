use crate::util::currency::converter::ConvertError;
use oximod::_error::oximod_error::OxiModError;
use redis;
use teloxide::RequestError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Teloxide API Error: {0}")]
    Teloxide(#[from] RequestError),

    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Gemini AI Error: {0}")]
    Gemini(#[from] gem_rs::errors::GemError),

    #[error("Currency Conversion Error: {0}")]
    CurrencyConversion(#[from] ConvertError),

    #[error("MongoDB Error: {0}")]
    MongoDb(#[from] OxiModError),

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] ParseError),

    #[error("Application Error: {0}")]
    Other(String),
}
impl From<&str> for MyError {
    fn from(s: &str) -> Self {
        MyError::Other(s.to_string())
    }
}
impl From<String> for MyError {
    fn from(s: String) -> Self {
        MyError::Other(s)
    }
}
