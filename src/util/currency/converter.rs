use std::{
    collections::HashMap,
    fs,
    sync::{Arc},
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::Mutex;

const API_URL: &str = "https://open.er-api.com/v6/latest/UAH";
const CACHE_DURATION_SECS: u64 = 24 * 60 * 60;
const CURRENCY_CONFIG_PATH: &str = "currencies.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLanguage {
    Russian,
    // English,
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("API returned an error: {0}")]
    ApiError(String),
    #[error("Cache lock poisoned")]
    CurrencyNotFound(String),
    #[error("Invalid currency format detected during parsing")]
    RateNotFound(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Failed to read currency config file '{0}': {1}")]
    ConfigFileReadError(String, std::io::Error),
    #[error("Failed to parse currency config file '{0}': {1}")]
    ConfigFileParseError(String, serde_json::Error),
}

#[derive(Deserialize, Debug, Clone)]
struct ApiResponse {
    result: String,
    base_code: String,
    rates: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct CachedRates {
    fetched_at: Instant,
    data: ApiResponse,
}

type Cache = Arc<Mutex<Option<CachedRates>>>;

#[derive(Deserialize, Debug, Clone)]
struct CurrencyConfig {
    code: String,
    symbol: String,
    flag: String,
    patterns: Vec<String>,
    one: String,
    few: String,
    many: String,
    #[warn(dead_code)] // TODO: English Support
    one_en: String,
    #[warn(dead_code)]
    many_en: String,
    is_target: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DetectedCurrency {
    amount: f64,
    currency_code: String,
}

static CURRENCY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*([a-zA-Zа-яА-Я€$₽₴£]+(?:[\.\s]*(?:рубл[ьяей]*|ruble[s]?|hryvnia[s]?|euro[s]?|dollar[s]?|pound[s]?))?)|([€$₽₴£])\s*(\d+(?:[.,]\d+)?)")
        .expect("Invalid Regex")
});

pub struct CurrencyConverter {
    cache: Cache,
    client: Client,
    currency_info: HashMap<String, CurrencyConfig>,
    target_currencies: Vec<String>,
    language: OutputLanguage,
}

fn get_plural_form(number: u64, one: &str, few: &str, many: &str) -> String {
    let last_two_digits = number % 100;
    let last_digit = number % 10;
    if last_two_digits >= 11 && last_two_digits <= 19 { many.to_string() }
    else if last_digit == 1 { one.to_string() }
    else if last_digit >= 2 && last_digit <= 4 { few.to_string() }
    else { many.to_string() }
}

impl CurrencyConverter {
    pub fn new(language: OutputLanguage) -> Result<Self, ConvertError> {
        let config_path_str = CURRENCY_CONFIG_PATH;
        let config_content = fs::read_to_string(config_path_str)
            .map_err(|e| ConvertError::ConfigFileReadError(config_path_str.to_string(), e))?;

        let currencies: Vec<CurrencyConfig> = serde_json::from_str(&config_content)
            .map_err(|e| ConvertError::ConfigFileParseError(config_path_str.to_string(), e))?;

        let mut currency_map = HashMap::new();
        let mut target_codes = Vec::new();

        for currency in currencies {
            if currency.is_target { target_codes.push(currency.code.clone()); }
            currency_map.insert(currency.code.clone(), currency);
        }

        if currency_map.is_empty() { eprintln!("Warning: No currency configurations loaded from {}.", config_path_str); }
        if target_codes.is_empty() { eprintln!("Warning: No target currencies defined in {}.", config_path_str); }

        Ok(CurrencyConverter {
            cache: Arc::new(Mutex::new(None)),
            client: Client::new(),
            currency_info: currency_map,
            target_currencies: target_codes,
            language,
        })
    }

    async fn fetch_rates(&self) -> Result<ApiResponse, ConvertError> {
        let response = self.client.get(API_URL).send().await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_else(|_| "Failed to get error body".to_string());
            return Err(ConvertError::ApiError(format!("API request failed with status {}: {}", status, text)));
        }
        let api_response = response.json::<ApiResponse>().await?;
        if api_response.result != "success" {
            return Err(ConvertError::ApiError(format!("API indicated failure. Result: {}", api_response.result)));
        }
        Ok(api_response)
    }

    async fn get_rates(&self) -> Result<ApiResponse, ConvertError> {
        let cache_guard = self.cache.lock().await;

        if let Some(cached_data) = &*cache_guard {
            if cached_data.fetched_at.elapsed() < Duration::from_secs(CACHE_DURATION_SECS) {
                let data_clone = cached_data.data.clone();
                return Ok(data_clone);
            }
        }

        drop(cache_guard);

        let rates_data = self.fetch_rates().await?;

        let mut cache_guard = self.cache.lock().await;

        *cache_guard = Some(CachedRates {
            fetched_at: Instant::now(),
            data: rates_data.clone(),
        });

        Ok(rates_data)
    }

    fn find_currency_info(&self, code: &str) -> Option<&CurrencyConfig> {
        self.currency_info.get(code)
    }

    fn find_currency_info_by_identifier(&self, identifier: &str) -> Option<&CurrencyConfig> {
        let lower_identifier = identifier.to_lowercase().replace(['.', ' '], "");
        let clean_identifier = if lower_identifier.starts_with("belarusian") && lower_identifier.contains("ruble") {
            "belarusianruble"
        } else if lower_identifier.starts_with("бел") && lower_identifier.contains("руб") {
            "белруб"
        } else {
            &lower_identifier
        };

        self.currency_info.values().find(|info| {
            info.patterns.iter().any(|p| p.replace(['.', ' '], "") == clean_identifier)
                || info.symbol == identifier
        })
    }

    fn convert_amount( amount: f64, from_code: &str, to_code: &str, rates_map: &HashMap<String, f64>, base_code: &str) -> Result<f64, ConvertError> {
        if from_code == to_code { return Ok(amount); }
        let rate_from = if from_code == base_code { 1.0 } else { *rates_map.get(from_code).ok_or_else(|| ConvertError::RateNotFound(from_code.to_string()))? };
        let rate_to = if to_code == base_code { 1.0 } else { *rates_map.get(to_code).ok_or_else(|| ConvertError::RateNotFound(to_code.to_string()))? };
        if rate_from == 0.0 { return Err(ConvertError::InternalError(format!("Rate for base currency '{}' is zero.", from_code))); }
        Ok(amount * (rate_to / rate_from))
    }

    fn format_conversion_result( &self, original: &DetectedCurrency, rates_data: &ApiResponse ) -> Result<String, ConvertError> {
        let mut result = String::new();
        let original_info = self.find_currency_info(&original.currency_code)
            .ok_or_else(|| ConvertError::CurrencyNotFound(original.currency_code.clone()))?;

        let original_word = match self.language {
            OutputLanguage::Russian => {
                let amount_int = original.amount.trunc() as u64;
                get_plural_form(amount_int, &original_info.one, &original_info.few, &original_info.many)
            }
            // OutputLanguage::English => {
            //     get_plural_form_en(original.amount, &original_info.one_en, &original_info.many_en)
            // }
        };
        result.push_str(&format!( "{} {:.2}{} {}\n\n", original_info.flag, original.amount, original_info.symbol, original_word ));

        for target_code in &self.target_currencies {
            if target_code == &original.currency_code { continue; }

            if let Some(target_info) = self.find_currency_info(target_code) {
                match Self::convert_amount( original.amount, &original.currency_code, target_code, &rates_data.rates, &rates_data.base_code ) {
                    Ok(converted_amount) => {
                        let word = match self.language {
                            OutputLanguage::Russian => {
                                let amount_int = converted_amount.trunc() as u64;
                                get_plural_form(amount_int, &target_info.one, &target_info.few, &target_info.many)
                            }
                            // OutputLanguage::English => {
                            //     get_plural_form_en(converted_amount, &target_info.one_en, &target_info.many_en)
                            // }
                        };
                        result.push_str(&format!( "{} {:.2}{} {}\n", target_info.flag, converted_amount, target_info.symbol, word ));
                    }
                    Err(ConvertError::RateNotFound(missing_code)) => {
                        eprintln!("Warning: Rate not found for '{}' in API response when converting from {}. Skipping.", missing_code, original.currency_code);
                    }
                    Err(e) => {
                        eprintln!("Warning: Conversion error from {} to {}: {}. Skipping.", original.currency_code, target_code, e);
                    }
                }
            } else {
                eprintln!("Critical Warning: Currency info not found for target code '{}' defined in target_currencies.", target_code);
            }
        }

        if result.ends_with('\n') { result.pop(); }
        Ok(result)
    }

    pub async fn process_text(&self, text: &str) -> Result<Vec<String>, ConvertError> {
        let detected_currencies = self.parse_text_for_currencies(text)?;
        if detected_currencies.is_empty() { return Ok(Vec::new()); }
        let rates_data = self.get_rates().await?;
        let mut results = Vec::new();
        for detected in detected_currencies {
            match self.format_conversion_result(&detected, &rates_data) {
                Ok(formatted) => results.push(formatted),
                Err(e) => eprintln!("Error formatting conversion for {:?}: {}", detected, e),
            }
        }
        Ok(results)
    }

    pub(crate) fn parse_text_for_currencies(&self, text: &str) -> Result<Vec<DetectedCurrency>, ConvertError> {
        let mut detected: Vec<DetectedCurrency> = Vec::new();
        for cap in CURRENCY_REGEX.captures_iter(text) {
            let (amount_str, identifier_str) = if let (Some(amount), Some(identifier)) = (cap.get(1), cap.get(2)) {
                (amount.as_str(), identifier.as_str().trim())
            } else if let (Some(symbol), Some(amount)) = (cap.get(3), cap.get(4)) {
                (amount.as_str(), symbol.as_str())
            } else { continue; };

            let amount_str_cleaned = amount_str.replace(',', ".");
            if let Ok(amount) = amount_str_cleaned.parse::<f64>() {
                if let Some(info) = self.find_currency_info_by_identifier(identifier_str) {
                    detected.push(DetectedCurrency { amount, currency_code: info.code.clone() });
                }
            } else {
                eprintln!("Warning: Failed to parse amount '{}'", amount_str);
            }
        }
        Ok(detected)
    }
}