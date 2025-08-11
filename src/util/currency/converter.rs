use std::{
    collections::HashMap,
    fs,
    sync::Arc,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::Mutex;

const CACHE_DURATION_SECS: u64 = 60 * 10;
const CURRENCY_CONFIG_PATH: &str = "currencies.json";
const COINBASE_API_URL: &str = "https://api.coinbase.com/v2/exchange-rates?currency=UAH";
const TONAPI_URL: &str = "https://tonapi.io/v2/rates";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLanguage {
    Russian,
    #[allow(dead_code)]
    English, // when
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("API returned an error: {0}")]
    #[allow(dead_code)]
    ApiError(String),
    #[error("Currency '{0}' not found in the configuration")]
    CurrencyNotFound(String),
    #[error("Rate for '{0}' not found in the combined API responses")]
    RateNotFound(String),
    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    InternalError(String),
    #[error("Failed to read currency config file '{0}': {1}")]
    ConfigFileReadError(String, std::io::Error),
    #[error("Failed to parse currency config file '{0}': {1}")]
    ConfigFileParseError(String, serde_json::Error),
    #[error("No rates could be fetched from any API")]
    NoRatesFetched,
    #[error("Failed to build regex from config: {0}")]
    #[allow(dead_code)]
    RegexBuildError(String),
}

fn build_regex_from_config() -> Result<String, ConvertError> {
    let config_content = fs::read_to_string(CURRENCY_CONFIG_PATH)
        .map_err(|e| ConvertError::ConfigFileReadError(CURRENCY_CONFIG_PATH.to_string(), e))?;

    let currencies: Vec<CurrencyConfig> = serde_json::from_str(&config_content)
        .map_err(|e| ConvertError::ConfigFileParseError(CURRENCY_CONFIG_PATH.to_string(), e))?;

    let mut all_patterns = Vec::new();
    let mut all_symbols = Vec::new();

    for currency in currencies {
        all_patterns.extend(currency.patterns.iter().cloned());
        if !currency.symbol.is_empty() {
            all_symbols.push(currency.symbol.clone());
        }
    }

    let escaped_patterns: Vec<String> = all_patterns.iter().map(|p| regex::escape(p)).collect();
    let escaped_symbols: Vec<String> = all_symbols.iter().map(|s| regex::escape(s)).collect();

    let patterns_part = escaped_patterns.join("|");
    let symbols_part = escaped_symbols.join("|");

    let regex_string = format!(
        r"(?i)\b(\d+(?:[.,]\d+)?)\s*\b({})\b|({})\s*\b(\d+(?:[.,]\d+)?)",
        patterns_part, symbols_part
    );

    Ok(regex_string)
}

static CURRENCY_REGEX: Lazy<Regex> = Lazy::new(|| {
    let regex_string = build_regex_from_config()
        .map_err(|e| e.to_string())
        .expect("FATAL: Could not build regex from currency config file.");
    Regex::new(&regex_string)
        .unwrap_or_else(|e| panic!("FATAL: Invalid regex generated from config: {}", e))
});

#[derive(Deserialize, Debug, Clone)]
struct CurrencyConfig {
    code: String,
    source: String,
    #[serde(default)]
    api_identifier: Option<String>,
    symbol: String,
    flag: String,
    patterns: Vec<String>,
    one: String,
    few: String,
    many: String,
    #[allow(dead_code)]
    one_en: String,
    #[allow(dead_code)]
    many_en: String,
    is_target: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DetectedCurrency {
    amount: f64,
    currency_code: String,
}

#[derive(Debug, Clone)]
struct CachedRates {
    fetched_at: Instant,
    rates: HashMap<String, f64>,
    #[allow(dead_code)]
    base_code: String,
}

type Cache = Arc<Mutex<Option<CachedRates>>>;

#[derive(Deserialize, Debug)]
struct CoinbaseResponse {
    data: CoinbaseData,
}
#[derive(Deserialize, Debug)]
struct CoinbaseData {
    currency: String,
    rates: HashMap<String, String>,
}
#[derive(Deserialize, Debug)]
struct TonApiResponse {
    rates: HashMap<String, TonRateEntry>,
}
#[derive(Deserialize, Debug, Clone)]
struct TonRateEntry {
    prices: HashMap<String, f64>,
}

pub struct CurrencyConverter {
    cache: Cache,
    client: Client,
    currency_info: HashMap<String, CurrencyConfig>,
    target_currencies: Vec<String>,
    #[allow(dead_code)]
    language: OutputLanguage, // when

    // for fucking ton api
    ton_tickers: Vec<String>,
    ton_addresses: Vec<String>,
    ton_ticker_to_code: HashMap<String, String>, // <"ton", "TON">
    ton_address_to_code: HashMap<String, String>, // <"EQ..NOT", "NOT">
}

fn get_plural_form(number: u64, one: &str, few: &str, many: &str) -> String {
    let last_two_digits = number % 100;
    let last_digit = number % 10;
    if (11..=19).contains(&last_two_digits) {
        many.to_string()
    } else if last_digit == 1 {
        one.to_string()
    } else if (2..=4).contains(&last_digit) {
        few.to_string()
    } else {
        many.to_string()
    }
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

        // for fucking ton api
        let mut ton_tickers = Vec::new();
        let mut ton_addresses = Vec::new();
        let mut ton_ticker_to_code = HashMap::new();
        let mut ton_address_to_code = HashMap::new();

        for currency in currencies {
            if currency.is_target {
                target_codes.push(currency.code.clone());
            }

            if currency.source == "tonapi" {
                if let Some(identifier) = &currency.api_identifier {
                    if identifier.len() > 10
                        && (identifier.starts_with("EQ") || identifier.starts_with("UQ"))
                    {
                        ton_addresses.push(identifier.clone());
                        ton_address_to_code.insert(identifier.clone(), currency.code.clone());
                    } else {
                        let lower_ticker = identifier.to_lowercase();
                        ton_tickers.push(lower_ticker.clone());
                        ton_ticker_to_code.insert(lower_ticker, currency.code.clone());
                    }
                }
            }
            currency_map.insert(currency.code.clone(), currency);
        }

        Ok(CurrencyConverter {
            cache: Arc::new(Mutex::new(None)),
            client: Client::new(),
            currency_info: currency_map,
            target_currencies: target_codes,
            language,
            ton_tickers,
            ton_addresses,
            ton_ticker_to_code,
            ton_address_to_code,
        })
    }

    async fn fetch_crypto_rates(&self) -> Result<HashMap<String, f64>, ConvertError> {
        let all_tokens = [self.ton_tickers.as_slice(), self.ton_addresses.as_slice()].concat();
        if all_tokens.is_empty() {
            return Ok(HashMap::new());
        }

        let tokens_str = all_tokens.join(",");
        let response = self
            .client
            .get(TONAPI_URL)
            .query(&[("tokens", &tokens_str), ("currencies", &"uah".to_string())])
            .send()
            .await?;

        let parsed = response.json::<TonApiResponse>().await?;

        let mut crypto_rates = HashMap::new();
        for (api_identifier, rate_entry) in parsed.rates {
            let mut code: Option<String> = None;

            if let Some(found_code) = self.ton_address_to_code.get(&api_identifier) {
                code = Some(found_code.clone());
            } else if let Some(found_code) =
                self.ton_ticker_to_code.get(&api_identifier.to_lowercase())
            {
                code = Some(found_code.clone());
            }

            if let Some(found_code) = code {
                if let Some(price_in_uah) = rate_entry.prices.get("UAH") {
                    crypto_rates.insert(found_code, *price_in_uah);
                }
            } else {
                // ???
                eprintln!(
                    "[DEBUG] Skipped unknown API identifier from TonAPI: {}",
                    api_identifier
                );
            }
        }
        Ok(crypto_rates)
    }

    async fn fetch_fiat_rates(&self) -> Result<HashMap<String, f64>, ConvertError> {
        let response = self.client.get(COINBASE_API_URL).send().await?;
        let parsed = response.json::<CoinbaseResponse>().await?;
        let mut rates = parsed
            .data
            .rates
            .into_iter()
            .filter_map(|(currency_code, rate_str)| {
                rate_str.parse::<f64>().ok().and_then(|rate_val| {
                    if rate_val != 0.0 {
                        Some((currency_code, 1.0 / rate_val))
                    } else {
                        None
                    }
                })
            })
            .collect::<HashMap<String, f64>>();
        rates.insert(parsed.data.currency, 1.0);
        Ok(rates)
    }

    async fn fetch_rates(&self) -> Result<CachedRates, ConvertError> {
        let (fiat_result, crypto_result) =
            tokio::join!(self.fetch_fiat_rates(), self.fetch_crypto_rates());

        let mut combined_rates = fiat_result.map_err(|e| {
            eprintln!("CRITICAL: Failed to fetch vital fiat rates: {}", e);
            e
        })?;

        if let Ok(crypto_rates) = crypto_result {
            combined_rates.extend(crypto_rates);
        }

        if combined_rates.is_empty() {
            return Err(ConvertError::NoRatesFetched);
        }

        Ok(CachedRates {
            fetched_at: Instant::now(),
            rates: combined_rates,
            base_code: "UAH".to_string(),
        })
    }

    async fn get_rates(&self) -> Result<CachedRates, ConvertError> {
        let mut cache_guard = self.cache.lock().await;
        if let Some(cached_data) = &*cache_guard {
            if cached_data.fetched_at.elapsed() < Duration::from_secs(CACHE_DURATION_SECS) {
                return Ok(cached_data.clone());
            }
        }

        let new_rates = self.fetch_rates().await?;

        *cache_guard = Some(new_rates.clone());
        Ok(new_rates)
    }

    pub fn parse_text_for_currencies(
        &self,
        text: &str,
    ) -> Result<Vec<DetectedCurrency>, ConvertError> {
        let mut detected: Vec<DetectedCurrency> = Vec::new();
        for cap in CURRENCY_REGEX.captures_iter(text) {
            let (amount_str, identifier_str) =
                if let (Some(amount), Some(identifier)) = (cap.get(1), cap.get(2)) {
                    (amount.as_str(), identifier.as_str().trim())
                } else if let (Some(symbol), Some(amount)) = (cap.get(3), cap.get(4)) {
                    (amount.as_str(), symbol.as_str())
                } else {
                    continue;
                };

            let amount_str_cleaned = amount_str.replace(',', ".");
            if let Ok(amount) = amount_str_cleaned.parse::<f64>() {
                if let Some(info) = self.find_currency_info_by_identifier(identifier_str) {
                    detected.push(DetectedCurrency {
                        amount,
                        currency_code: info.code.clone(),
                    });
                }
            }
        }
        Ok(detected)
    }

    fn find_currency_info(&self, code: &str) -> Option<&CurrencyConfig> {
        self.currency_info.get(code)
    }

    fn find_currency_info_by_identifier(&self, identifier: &str) -> Option<&CurrencyConfig> {
        let lower_identifier = identifier.to_lowercase().replace(['.', ' '], "");
        self.currency_info.values().find(|info| {
            info.patterns
                .iter()
                .any(|p| p.to_lowercase().replace(['.', ' '], "") == lower_identifier)
                || info.symbol == identifier
        })
    }

    fn convert_amount(
        amount: f64,
        from_code: &str,
        to_code: &str,
        rates_map: &HashMap<String, f64>,
    ) -> Result<f64, ConvertError> {
        if from_code == to_code {
            return Ok(amount);
        }
        let rate_from = *rates_map
            .get(from_code)
            .ok_or_else(|| ConvertError::RateNotFound(from_code.to_string()))?;

        let rate_to = *rates_map
            .get(to_code)
            .ok_or_else(|| ConvertError::RateNotFound(to_code.to_string()))?;

        Ok(amount * rate_from / rate_to)
    }

    fn format_conversion_result(
        &self,
        original: &DetectedCurrency,
        rates_data: &CachedRates,
    ) -> Result<String, ConvertError> {
        let mut result = String::new();
        let original_info = self
            .find_currency_info(&original.currency_code)
            .ok_or_else(|| ConvertError::CurrencyNotFound(original.currency_code.clone()))?;

        let original_word = get_plural_form(
            original.amount.trunc() as u64,
            &original_info.one,
            &original_info.few,
            &original_info.many,
        );

        result.push_str(&format!(
            "{} {:.2}{} {}\n\n",
            original_info.flag, original.amount, original_info.symbol, original_word
        ));

        for target_code in &self.target_currencies {
            if target_code == &original.currency_code {
                continue;
            }

            if let Some(target_info) = self.find_currency_info(target_code) {
                match Self::convert_amount(
                    original.amount,
                    &original.currency_code,
                    target_code,
                    &rates_data.rates,
                ) {
                    Ok(converted_amount) => {
                        let word = get_plural_form(
                            converted_amount.trunc() as u64,
                            &target_info.one,
                            &target_info.few,
                            &target_info.many,
                        );

                        result.push_str(&format!(
                            "{} {:.2}{} {}\n",
                            target_info.flag, converted_amount, target_info.symbol, word
                        ));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Conversion error from {} to {}: {}. Skipping.",
                            original.currency_code, target_code, e
                        );
                    }
                }
            }
        }
        Ok(result.trim_end().to_string())
    }

    pub async fn process_text(&self, text: &str) -> Result<Vec<String>, ConvertError> {
        let detected_currencies = self.parse_text_for_currencies(text)?;
        if detected_currencies.is_empty() {
            return Ok(Vec::new());
        }
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
}
