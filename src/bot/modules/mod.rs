pub mod cobalt;
pub mod currency;
pub mod registry;
pub mod whisper;

use crate::errors::MyError;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use teloxide::{prelude::*, types::InlineKeyboardMarkup};

#[derive(Clone, Debug)]
pub struct Owner {
    pub id: String,
    pub r#type: String, // user, group only
}

#[async_trait]
pub trait ModuleSettings:
    Sized + Default + Serialize + DeserializeOwned + Debug + Send + Sync
{
}

#[async_trait]
pub trait Module: Send + Sync {
    fn key(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    async fn get_settings_ui(
        &self,
        owner: &Owner,
    ) -> Result<(String, InlineKeyboardMarkup), MyError>;

    async fn handle_callback(
        &self,
        bot: Bot,
        q: &CallbackQuery,
        owner: &Owner,
        data: &str,
    ) -> Result<(), MyError>;

    fn enabled_for(&self, owner_type: &str) -> bool;

    fn factory_settings(&self) -> Result<serde_json::Value, MyError>;
}
