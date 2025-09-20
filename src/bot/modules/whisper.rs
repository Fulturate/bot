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
pub struct WhisperSettings {
    pub enabled: bool,
}

impl Default for WhisperSettings {
    fn default() -> Self {
        Self {
            enabled: false,
        }
    }
}

impl ModuleSettings for WhisperSettings {}

pub struct WhisperModule;

#[async_trait]
impl Module for WhisperModule {
    fn key(&self) -> &'static str {
        "whisper"
    }
    fn description(&self) -> &'static str {
        "Настройки для Whisper System"
    }

    async fn get_settings_ui(
        &self,
        owner: &Owner,
    ) -> Result<(String, InlineKeyboardMarkup), MyError> {
        let settings: WhisperSettings = Settings::get_module_settings(owner, self.key()).await?;

        let text = format!(
            "⚙️ <b>Настройки модуля</b>: {}\n\nСтатус: {}",
            self.description(),
            if settings.enabled {
                "✅ Включен"
            } else {
                "❌ Выключен"
            }
        );

        let toggle_button = InlineKeyboardButton::callback(
            if settings.enabled {
                "Выключить модуль"
            } else {
                "Включить модуль"
            },
            format!("{}:settings:toggle_module", self.key()),
        );

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![toggle_button],
            vec![InlineKeyboardButton::callback(
                "⬅️ Назад",
                format!("settings_back:{}:{}", owner.r#type, owner.id),
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
    ) -> Result<(), MyError> {
        let Some(message) = &q.message else {
            return Ok(());
        };

        let Some(message) = message.regular_message() else {
            return Ok(());
        };

        let parts: Vec<_> = data.split(':').collect();

        if parts.len() == 1 && parts[0] == "toggle_module" {
            let mut settings: WhisperSettings =
                Settings::get_module_settings(owner, self.key()).await?;
            settings.enabled = !settings.enabled;
            Settings::update_module_settings(owner, self.key(), settings).await?;

            let (text, keyboard) = self.get_settings_ui(owner).await?;
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            return Ok(());
        }

        if parts.len() < 3 || parts[0] != "set" {
            bot.answer_callback_query(q.id.clone()).await?;
            return Ok(());
        }

        let settings: WhisperSettings = Settings::get_module_settings(owner, self.key()).await?;

        Settings::update_module_settings(owner, self.key(), settings).await?;

        let (text, keyboard) = self.get_settings_ui(owner).await?;
        bot.edit_message_text(message.chat.id, message.id, text)
            .reply_markup(keyboard)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;

        Ok(())
    }

    fn enabled_for(&self, owner_type: &str) -> bool {
        owner_type == "user" // user
    }

    fn factory_settings(&self) -> Result<serde_json::Value, MyError> {
        let factory_settings = WhisperSettings {
            enabled: true,
        };
        Ok(serde_json::to_value(factory_settings)?)
    }
}
