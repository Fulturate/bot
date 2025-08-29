use std::{
    collections::HashMap,
    fs,
    sync::Arc,
    time::{Duration, Instant},
};

use super::structs::WORD_VALUES;
use crate::db::schemas::CurrenciesFunctions;
use crate::db::schemas::group::Group;
use crate::db::schemas::user::User;
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use oximod::Model;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use teloxide::prelude::InlineQuery;
use teloxide::types::Chat;
use thiserror::Error;
use tokio::sync::Mutex;
use crate::util::errors::MyError;

const CACHE_DURATION_SECS: u64 = 60 * 10;
pub const CURRENCY_CONFIG_PATH: &str = "currencies.json";
const COINBASE_API_URL: &str = "https://api.coinbase.com/v2/exchange-rates?currency=UAH";
const TONAPI_URL: &str = "https://tonapi.io/v2/rates";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLanguage {
    Russian,
    #[allow(dead_code)]
    English, // when
}

#[derive(Default)]
struct ParseState {
    total: f64,
    current_chunk: f64,
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

    let currencies: Vec<CurrencyStruct> = serde_json::from_str(&config_content)
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

    let multiplier_words_part: String = WORD_VALUES
        .iter()
        .filter(|(_, info)| info.is_multiplier)
        .map(|(word, _)| regex::escape(word))
        .collect::<Vec<String>>()
        .join("|");

    let number_words: Vec<String> = WORD_VALUES.keys().map(|s| regex::escape(s)).collect();

    let number_suffixes = r"к|k|м|m|б|b|т|t|тыс|млн|млрд|трлн|kk|кк";
    let repeatable_digits_part = format!(r"(?:[\d.,_ \t]*(?:[ \t]*(?:{number_suffixes}))?)+");

    let number_pattern_any = format!(
        r"(?:{}\b|(?:(?:{})\b[ \t]*)+)",
        repeatable_digits_part,
        number_words.join("|")
    );

    let regex_string = format!(
        concat!(
            r"(?i)(?:^|\s)(?:",
            r"({digits})[ \t]+({multiplier})[ \t]*({word_patterns})\b",
            r"|",
            r"(?:({multiplier})[ \t]+)?({number})[ \t]*({word_patterns})\b",
            r"|",
            r"({symbols})[ \t]*({number})\b",
            r"|",
            r"({number})[ \t]*({symbols})",
            r")",
        ),
        digits = repeatable_digits_part,
        multiplier = multiplier_words_part,
        number = number_pattern_any,
        word_patterns = patterns_part,
        symbols = symbols_part
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

// for combined values like 2k2k2k2k ton
static COMPONENT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d+(?:[.,]\d+)?)\s*(кк|kk|k|к|m|м|b|б|t|т|тыс|млн|млрд|трлн)").unwrap()
});
static INFIX_K_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+(?:[.,]\d+)?)[kк](\d{1,3})$").unwrap());

pub async fn is_currency_query(q: InlineQuery) -> bool {
    CURRENCY_REGEX.is_match(&q.query)
}

pub fn get_all_currency_codes(config_file: String) -> Result<Vec<CurrencyStruct>, ConvertError> {
    let mut codes: Vec<CurrencyStruct> = vec![];

    let config_content = fs::read_to_string(config_file.clone())
        .map_err(|e| ConvertError::ConfigFileReadError(config_file.to_string(), e))?;
    let currencies: Vec<CurrencyStruct> = serde_json::from_str(&config_content)
        .map_err(|e| ConvertError::ConfigFileParseError(config_file.to_string(), e))?;

    currencies
        .iter()
        .for_each(|currency| codes.push(currency.clone()));

    Ok(codes)
}

pub fn get_default_currencies() -> Result<Vec<CurrencyStruct>, MyError> {
    let all_codes = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap())?;

    let necessary_codes = all_codes
        .iter()
        .filter(|c| {
            ["uah", "rub", "usd", "byn", "eur", "ton"].contains(&c.code.to_lowercase().as_str())
        })
        .cloned()
        .collect::<Vec<CurrencyStruct>>();

    Ok(necessary_codes)
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrencyStruct {
    pub code: String,
    pub source: String,
    #[serde(default)]
    pub api_identifier: Option<String>,
    pub symbol: String,
    pub flag: String,
    pub patterns: Vec<String>,
    pub one: String,
    pub few: String,
    pub many: String,
    #[allow(dead_code)]
    pub one_en: String,
    #[allow(dead_code)]
    pub many_en: String,
    pub is_target: bool,
}

pub struct CurrencyConverter {
    cache: Cache,
    client: Client,
    currency_info: HashMap<String, CurrencyStruct>,
    // target_currencies: Vec<String>,
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
        let currencies: Vec<CurrencyStruct> = serde_json::from_str(&config_content)
            .map_err(|e| ConvertError::ConfigFileParseError(config_path_str.to_string(), e))?;

        let mut currency_map = HashMap::new();
        // let mut target_codes = Vec::new();

        // for fucking ton api
        let mut ton_tickers = Vec::new();
        let mut ton_addresses = Vec::new();
        let mut ton_ticker_to_code = HashMap::new();
        let mut ton_address_to_code = HashMap::new();

        for currency in currencies {
            // if currency.is_target {
            //     target_codes.push(currency.code.clone());
            // }

            if currency.source == "tonapi"
                && let Some(identifier) = &currency.api_identifier
            {
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
            currency_map.insert(currency.code.clone(), currency);
        }

        Ok(CurrencyConverter {
            cache: Arc::new(Mutex::new(None)),
            client: Client::new(),
            currency_info: currency_map,
            // target_currencies: target_codes,
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

        parsed
            .rates
            .iter()
            .for_each(|(api_identifier, rate_entry)| {
                let mut code: Option<String> = None;

                if let Some(found_code) = self.ton_address_to_code.get(api_identifier) {
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
                    debug!(
                        "Skipped unknown API identifier from TonAPI: {}",
                        api_identifier
                    );
                }
            });
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
            error!("CRITICAL: Failed to fetch vital fiat rates: {}", e);
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
        if let Some(cached_data) = &*cache_guard
            && cached_data.fetched_at.elapsed() < Duration::from_secs(CACHE_DURATION_SECS)
        {
            return Ok(cached_data.clone());
        }

        let new_rates = self.fetch_rates().await?;

        *cache_guard = Some(new_rates.clone());
        Ok(new_rates)
    }

    pub fn parse_number_words(text: &str) -> Option<f64> {
        let state = text
            .split_whitespace()
            .filter_map(|word| WORD_VALUES.get(word.to_lowercase().as_str()))
            .fold(ParseState::default(), |mut state, info| {
                if info.is_multiplier {
                    let chunk_to_add = if state.current_chunk == 0.0 {
                        1.0
                    } else {
                        state.current_chunk
                    };
                    state.total += chunk_to_add * info.value;
                    state.current_chunk = 0.0;
                } else if info.value == 100.0 && state.current_chunk > 0.0 {
                    state.current_chunk *= info.value;
                } else {
                    state.current_chunk += info.value;
                }
                state
            });

        let result = state.total + state.current_chunk;

        if result > 0.0 {
            Some(result)
        } else if text
            .split_whitespace()
            .any(|w| w == "ноль" || w == "нуль" || w == "zero")
        {
            Some(0.0)
        } else {
            None
        }
    }

    pub fn parse_text_for_currencies(
        &self,
        text: &str,
    ) -> Result<Vec<DetectedCurrency>, ConvertError> {
        let parse_amount_or_words = |amount_str: &str| -> Option<f64> {
            let first_char = amount_str.chars().next();
            if first_char.is_some_and(|c| c.is_alphabetic() && c.to_lowercase().next() != Some('a'))
            {
                Self::parse_number_words(amount_str)
            } else {
                Self::parse_amount_with_suffix(amount_str)
            }
        };

        let currencies = CURRENCY_REGEX
            .captures_iter(text)
            .filter_map(|cap| {
                let (amount, identifier_str) =
                    // {num} {multiplier} {symbol}
                    if let (Some(num_str), Some(multiplier_match), Some(identifier)) =
                        (cap.get(1), cap.get(2), cap.get(3))
                    {
                        let base_amount = Self::parse_amount_with_suffix(num_str.as_str().trim())?;
                        let multiplier_value = WORD_VALUES
                            .get(multiplier_match.as_str().to_lowercase().as_str())?
                            .value;
                        Some((base_amount * multiplier_value, identifier.as_str().trim()))
                    }

                    // {num}{symbol} with hidden {multiplier}
                    else if let (Some(num_str), Some(identifier)) = (cap.get(5), cap.get(6)) {
                        let base_amount = parse_amount_or_words(num_str.as_str().trim())?;

                        // check multiplier
                        let final_amount = if let Some(multiplier_match) = cap.get(4) {
                            let multiplier_value = WORD_VALUES
                                .get(multiplier_match.as_str().to_lowercase().as_str())?
                                .value;
                            base_amount * multiplier_value
                        } else {
                            base_amount
                        };

                        Some((final_amount, identifier.as_str().trim()))
                    }
                    // {symbol}{num}
                    else if let (Some(identifier), Some(amount_str)) = (cap.get(7), cap.get(8)) {
                        let amount = parse_amount_or_words(amount_str.as_str().trim())?;
                        Some((amount, identifier.as_str()))
                    }
                    // {num}{symbol}
                    else if let (Some(amount_str), Some(identifier)) = (cap.get(9), cap.get(10)) {
                        let amount = parse_amount_or_words(amount_str.as_str().trim())?;
                        Some((amount, identifier.as_str()))
                    } else {
                        None
                    }?;

                let info = self.find_currency_info_by_identifier(identifier_str)?;

                Some(DetectedCurrency {
                    amount,
                    currency_code: info.code.clone(),
                })
            })
            .collect();

        Ok(currencies)
    }

    pub fn parse_amount_with_suffix(amount_str: &str) -> Option<f64> {
        let s = amount_str
            .trim()
            .to_lowercase()
            .replace(['_', ' '], "")
            .replace(',', ".");

        if s.is_empty() {
            return None;
        }

        let number_part_str = if let Some(last_dot_pos) = s.rfind('.') {
            let after_last_dot = &s[last_dot_pos + 1..];
            let is_thousand_separator = !after_last_dot.chars().any(|c| !c.is_ascii_digit())
                && after_last_dot.len() == 3
                && s.chars().filter(|&c| c == '.').count() > 0;

            if is_thousand_separator && s.split('.').all(|part| part.len() <= 3 || part.is_empty())
            {
                s.replace('.', "")
            } else {
                let (before_last_dot, remaining_part) = s.split_at(last_dot_pos);
                before_last_dot.replace('.', "") + remaining_part
            }
        } else {
            s
        };

        let get_multiplier = |suffix: &str| -> Option<f64> {
            match suffix {
                "к" | "k" | "тыс" => Some(1_000.0),
                "м" | "m" | "млн" | "кк" | "kk" => Some(1_000_000.0),
                "б" | "b" | "млрд" => Some(1_000_000_000.0),
                "т" | "t" | "трлн" => Some(1_000_000_000_000.0),
                _ => None,
            }
        };

        // single pattern check ($1k200)
        if let Some(caps) = INFIX_K_RE.captures(&number_part_str) {
            let before_k_str = caps.get(1).unwrap().as_str();
            let after_k_str = caps.get(2).unwrap().as_str();

            return Self::parse_amount_with_suffix(before_k_str).and_then(|before_val| {
                after_k_str
                    .parse::<f64>()
                    .ok()
                    .map(|after_val| before_val * 1000.0 + after_val)
            });
        }

        // multiple pattern check ($1m2k300)
        COMPONENT_RE
            .captures_iter(&number_part_str)
            .try_fold((0.0, 0), |(current_total, last_end), cap| {
                let full_match = cap.get(0).unwrap();
                if full_match.start() != last_end {
                    return Err("Invalid character between components");
                }
                let num_str = cap.get(1).unwrap().as_str();
                let suffix_str = cap.get(2).unwrap().as_str();
                num_str
                    .parse::<f64>()
                    .ok()
                    .and_then(|num| get_multiplier(suffix_str).map(|mult| num * mult))
                    .map(|value| (current_total + value, full_match.end()))
                    .ok_or("Failed to parse component")
            })
            .ok()
            .and_then(|(current_total, full_match_end)| {
                let tail = &number_part_str[full_match_end..];
                if tail.is_empty() {
                    Some(current_total)
                } else {
                    if full_match_end > 0 && tail.chars().any(|c| c.is_alphabetic()) {
                        return None;
                    }
                    tail.parse::<f64>()
                        .ok()
                        .map(|tail_val| current_total + tail_val)
                }
            })
            .or_else(|| number_part_str.parse::<f64>().ok())
    }

    fn find_currency_info(&self, code: &str) -> Option<&CurrencyStruct> {
        self.currency_info.get(code)
    }

    fn find_currency_info_by_identifier(&self, identifier: &str) -> Option<&CurrencyStruct> {
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
        target_codes: &[String],
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

        for target_code in target_codes {
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
                            "{} {:.5}{} {}\n",
                            target_info.flag, converted_amount, target_info.symbol, word
                        ));
                    }
                    Err(e) => {
                        warn!(
                            "Conversion error from {} to {}: {}. Skipping.",
                            original.currency_code, target_code, e
                        );
                    }
                }
            }
        }
        Ok(result.trim_end().to_string())
    }

    pub async fn process_text(&self, text: &str, chat: &Chat) -> Result<Vec<String>, ConvertError> {
        let chat_id_str = chat.id.to_string();
        let target_codes = if chat.is_private() {
            match User::find_one(mongodb::bson::doc! { "user_id": chat_id_str }).await {
                Ok(Some(user)) => user
                    .get_currencies()
                    .iter()
                    .map(|c| c.code.clone())
                    .collect(),
                _ => Vec::new(),
            }
        } else {
            match Group::find_one(mongodb::bson::doc! { "group_id": chat_id_str }).await {
                Ok(Some(group)) => group
                    .get_currencies()
                    .iter()
                    .map(|c| c.code.clone())
                    .collect(),
                _ => Vec::new(),
            }
        };

        if target_codes.is_empty() {
            return Ok(Vec::new());
        }

        let detected_currencies = self.parse_text_for_currencies(text)?;
        if detected_currencies.is_empty() {
            return Ok(Vec::new());
        }

        let rates_data = self.get_rates().await?;
        let mut results = Vec::new();
        for detected in detected_currencies {
            match self.format_conversion_result(&detected, &rates_data, &target_codes) {
                Ok(formatted) => results.push(formatted),
                Err(e) => error!("Error formatting conversion for {:?}: {}", detected, e),
            }
        }
        Ok(results)
    }
}
