use crate::{db::schemas::CurrencyStruct, util::currency::converter::ConvertError};
use std::fs;

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
