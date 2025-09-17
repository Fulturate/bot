use crate::{
    bot::modules::{Module, ModuleSettings, Owner},
    core::{db::schemas::settings::Settings, services::cobalt::VideoQuality},
    errors::MyError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CobaltSettings {
    pub enabled: bool,
    pub video_quality: VideoQuality,
    pub attribution: bool,
}

impl Default for CobaltSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            video_quality: VideoQuality::Q1080,
            attribution: false,
        }
    }
}

impl ModuleSettings for CobaltSettings {}

pub struct CobaltModule;

#[async_trait]
impl Module for CobaltModule {
    fn key(&self) -> &'static str {
        "cobalt"
    }
    fn description(&self) -> &'static str {
        "Настройки для Cobalt Downloader"
    }

    async fn get_settings_ui(
        &self,
        owner: &Owner,
    ) -> Result<(String, InlineKeyboardMarkup), MyError> {
        let settings: CobaltSettings = Settings::get_module_settings(owner, self.key()).await?;

        let text = format!(
            "⚙️ Настройки модуля: {}\n\nСтатус: {}",
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

        let quality_options = [
            VideoQuality::Q720,
            VideoQuality::Q1080,
            VideoQuality::Q1440,
            VideoQuality::Max,
        ];
        let quality_buttons = quality_options
            .iter()
            .map(|q| {
                let display_text = if settings.video_quality == *q {
                    format!("• {}p •", q.as_str())
                } else {
                    format!("{}p", q.as_str())
                };
                let cb_data = format!("{}:settings:set:quality:{}", self.key(), q.as_str());
                InlineKeyboardButton::callback(display_text, cb_data)
            })
            .collect::<Vec<_>>();

        let attr_text = if settings.attribution {
            "Атрибуция: Вкл ✅"
        } else {
            "Атрибуция: Выкл ❌"
        };
        let attr_cb = format!(
            "{}:settings:set:attribution:{}",
            self.key(),
            !settings.attribution
        );

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![toggle_button],
            vec![InlineKeyboardButton::callback("Качество видео", "noop")],
            quality_buttons,
            vec![InlineKeyboardButton::callback(attr_text, attr_cb)],
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
            let mut settings: CobaltSettings =
                Settings::get_module_settings(owner, self.key()).await?;
            settings.enabled = !settings.enabled;
            Settings::update_module_settings(owner, self.key(), settings).await?;

            let (text, keyboard) = self.get_settings_ui(owner).await?;
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .await?;
            return Ok(());
        }

        if parts.len() < 3 || parts[0] != "set" {
            bot.answer_callback_query(q.id.clone()).await?;
            return Ok(());
        }

        let mut settings: CobaltSettings = Settings::get_module_settings(owner, self.key()).await?;

        match (parts[1], parts[2]) {
            ("quality", val) => {
                settings.video_quality = VideoQuality::from_str(val);
            }
            ("attribution", val) => {
                settings.attribution = val.parse().unwrap_or(false);
            }
            _ => {}
        }

        Settings::update_module_settings(owner, self.key(), settings).await?;

        let (text, keyboard) = self.get_settings_ui(owner).await?;
        bot.edit_message_text(message.chat.id, message.id, text)
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    fn enabled_for(&self, owner_type: &str) -> bool {
        owner_type == "user" // user
    }

    fn factory_settings(&self) -> Result<serde_json::Value, MyError> {
        let factory_settings = CobaltSettings {
            enabled: true,
            video_quality: VideoQuality::Q1080,
            attribution: false,
        };
        Ok(serde_json::to_value(factory_settings)?)
    }
}
