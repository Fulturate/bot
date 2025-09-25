use crate::{
    bot::modules::{Module, ModuleSettings, Owner},
    core::db::schemas::settings::Settings,
    errors::MyError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Default)]
pub struct TranslateSettings {
    pub enabled: bool,
    // TODO: save language
}

impl ModuleSettings for TranslateSettings {}

pub struct TranslateModule;

#[async_trait]
impl Module for TranslateModule {
    fn key(&self) -> &'static str {
        "translate"
    }

    fn name(&self) -> &'static str {
        "Translate"
    }

    fn description(&self) -> &'static str {
        "Протестировать можно через inlin'ы: \"@fulturatebot *слова для перевода*\"" // todo: finish this
    }

    async fn get_settings_ui(
        &self,
        owner: &Owner,
        commander_id: u64,
    ) -> Result<(String, InlineKeyboardMarkup), MyError> {
        let settings: TranslateSettings = Settings::get_module_settings(owner, self.key()).await?;

        let text = format!(
            "⚙️ <b>Настройки модуля</b>: {}\n<blockquote>{}</blockquote>\nСтатус: {}",
            self.name(),
            self.description(),
            if settings.enabled { "✅ Включен" } else { "❌ Выключен" }
        );

        let toggle_button = InlineKeyboardButton::callback(
            if settings.enabled { "Выключить модуль" } else { "Включить модуль" },
            format!("{}:settings:toggle_module:{}", self.key(), commander_id),
        );

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![toggle_button],
            vec![InlineKeyboardButton::callback(
                "⬅️ Назад",
                format!("settings_back:{}:{}:{}", owner.r#type, owner.id, commander_id),
            )],
        ]);

        Ok((text, keyboard))
    }

    async fn handle_callback(
        &self,
        bot: Bot,
        q: &CallbackQuery,
        owner: &Owner,
        data: &str,
        commander_id: u64,
    ) -> Result<(), MyError> {
        let Some(message) = &q.message else { return Ok(()); };
        let Some(message) = message.regular_message() else { return Ok(()); };

        let parts: Vec<_> = data.split(':').collect();

        if parts.len() == 1 && parts[0] == "toggle_module" {
            let mut settings: TranslateSettings =
                Settings::get_module_settings(owner, self.key()).await?;
            settings.enabled = !settings.enabled;
            Settings::update_module_settings(owner, self.key(), settings).await?;

            let (text, keyboard) = self.get_settings_ui(owner, commander_id).await?;
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            return Ok(());
        }

        bot.answer_callback_query(q.id.clone()).await?;

        Ok(())
    }

    fn designed_for(&self, owner_type: &str) -> bool {
        owner_type == "user"
    }

    async fn is_enabled(&self, owner: &Owner) -> bool {
        if !self.designed_for(&owner.r#type) {
            return false;
        }
        let settings: TranslateSettings = Settings::get_module_settings(owner, self.key()).await.unwrap(); // god of unwraps
        settings.enabled
    }

    fn factory_settings(&self) -> Result<serde_json::Value, MyError> {
        let factory_settings = TranslateSettings { enabled: true };
        Ok(serde_json::to_value(factory_settings)?)
    }
}