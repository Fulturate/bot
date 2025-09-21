use crate::core::services::currency::converter::ConvertError;
use ccobalt::model::error::CobaltError;
use mongodb::bson;
use oximod::_error::oximod_error::OxiModError;
use redis;
use std::string::FromUtf8Error;
use teloxide::RequestError;
use thiserror::Error;
use translators::Error as TranslatorError;
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

    #[error("Bson serialization error: {0}")]
    Bson(#[from] bson::ser::Error),

    #[error("Module '{0}' not found")]
    ModuleNotFound(String),

    #[error("Uuid error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Cobalt error: {0}")]
    CobaltError(#[from] CobaltError),

    #[error("BSON OID error: {0}")]
    BsonOid(#[from] bson::oid::Error),

    #[error("Base64 decoding error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Translation service error: {0}")]
    Translation(#[from] TranslatorError),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("User not found")]
    UserNotFound,
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
