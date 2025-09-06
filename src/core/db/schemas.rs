pub mod group;
pub mod settings;
pub mod user;

use crate::core::db::schemas::settings::ModuleSettings;
use crate::errors::MyError;
use crate::util::currency::converter::CurrencyStruct;
use async_trait::async_trait;
use mongodb::results::UpdateResult;
use oximod::_error::oximod_error::OxiModError;

#[async_trait]
pub trait BaseFunctions: Sized {
    async fn find_by_id(id: String) -> Result<Option<Self>, OxiModError>;
    async fn create_with_id(id: String) -> Result<Self, OxiModError>;
}

#[async_trait]
pub trait CurrenciesFunctions: Sized {
    fn get_id(&self) -> &str;
    fn get_currencies(&self) -> &Vec<CurrencyStruct>;
    async fn add_currency(id: &str, currency: &CurrencyStruct)
    -> Result<UpdateResult, OxiModError>;
    async fn remove_currency(id: &str, currency: &str) -> Result<UpdateResult, OxiModError>;
}

#[async_trait]
pub trait SettingsRepo {
    async fn get_or_create(owner_id: &str, owner_type: &str) -> Result<Self, MyError>
    where
        Self: Sized;

    async fn update_module<F>(
        owner_id: &str,
        owner_type: &str,
        module_key: &str,
        modifier: F,
    ) -> Result<Self, MyError>
    where
        Self: Sized,
        F: FnOnce(&mut ModuleSettings) + Send;

    fn modules_mut(&mut self) -> &mut Vec<ModuleSettings>;
}
