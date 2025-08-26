pub mod group;
pub mod user;

use crate::util::currency::converter::CurrencyStruct;
use async_trait::async_trait;
use oximod::_error::oximod_error::OxiModError;

#[async_trait]
pub trait BaseFunctions: Sized {
    async fn find_by_id(id: String) -> Result<Option<Self>, OxiModError>;
    async fn create_with_id(id: String) -> Result<Self, OxiModError>;
}

#[allow(dead_code)]
pub trait CurrenciesFunctions {
    fn get_id(&self) -> &str;
    fn get_currencies(&self) -> &Vec<CurrencyStruct>;
    fn id_field_name() -> &'static str;
}
