use crate::{
    bot::modules::{Module, ModuleSettings, Owner},
    core::{
        db::schemas::{group::Group, settings::Settings, user::User},
        services::{
            currencier::handle_currency_update,
            currency::converter::{get_all_currency_codes, get_default_currencies, CURRENCY_CONFIG_PATH},
        },
    },
    errors::MyError,
    util::paginator::{ItemsBuild, Paginator},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencySettings {
    pub enabled: bool,
    pub selected_codes: Vec<String>,
}

impl Default for CurrencySettings {
    fn default() -> Self {
        let default_currencies = get_default_currencies()
            .map(|currencies| currencies.into_iter().map(|c| c.code).collect::<Vec<String>>())
            .unwrap_or_else(|_| vec!["usd".to_string(), "eur".to_string()]);

        Self {
            enabled: true,
            selected_codes: default_currencies,
        }
    }
}

impl ModuleSettings for CurrencySettings {}

pub struct CurrencyModule;

#[async_trait]
impl Module for CurrencyModule {
    fn key(&self) -> &'static str {
        "currency"
    }

    fn name(&self) -> &'static str {
        "Конвертер валют"
    }

    fn description(&self) -> &'static str {
        "Возможность конвертировать валюты" // todo change this shit
    }

    async fn get_settings_ui(
        &self,
        owner: &Owner,
        commander_id: u64,
    ) -> Result<(String, InlineKeyboardMarkup), MyError> {
        self.get_paged_settings_ui(owner, 0, commander_id).await
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
            let mut settings: CurrencySettings =
                Settings::get_module_settings(owner, self.key()).await?;
            settings.enabled = !settings.enabled;
            if settings.enabled && settings.selected_codes.is_empty() {
                settings.selected_codes = vec![
                    "UAH".to_string(), "RUB".to_string(), "USD".to_string(),
                    "BYN".to_string(), "EUR".to_string(), "TON".to_string(),
                ];
            }
            Settings::update_module_settings(owner, self.key(), settings).await?;

            let (text, keyboard) = self.get_paged_settings_ui(owner, 0, commander_id).await?;
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            return Ok(());
        }

        if parts.len() == 2 && parts[0] == "page" {
            let page = parts[1].parse::<usize>().unwrap_or(0);
            let (text, keyboard) = self.get_paged_settings_ui(owner, page, commander_id).await?;
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
            return Ok(());
        }

        if parts.len() == 2 && parts[0] == "toggle" {
            let currency_code = parts[1].to_string();
            let mut settings: CurrencySettings =
                Settings::get_module_settings(owner, self.key()).await?;
            if let Some(pos) = settings.selected_codes.iter().position(|c| *c == currency_code) {
                settings.selected_codes.remove(pos);
            } else {
                settings.selected_codes.push(currency_code);
            }
            Settings::update_module_settings(owner, self.key(), settings).await?;
            let (text, keyboard) = self.get_paged_settings_ui(owner, 0, commander_id).await?; // TODO: сохранить текущую страницу
            bot.edit_message_text(message.chat.id, message.id, text)
                .reply_markup(keyboard)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?;
        } else {
            bot.answer_callback_query(q.id.clone()).await?;
        }
        Ok(())
    }

    fn designed_for(&self, _owner_type: &str) -> bool {
        true // all
    }

    async fn is_enabled(&self, owner: &Owner) -> bool {
        if !self.designed_for(&*owner.r#type) {
            return false;
        }
        let settings: CurrencySettings = Settings::get_module_settings(owner, self.key()).await.unwrap(); // god of unwraps
        settings.enabled
    }

    fn factory_settings(&self) -> Result<serde_json::Value, MyError> {
        let factory_settings = CurrencySettings {
            enabled: true,
            selected_codes: vec![
                "UAH".to_string(), "RUB".to_string(), "USD".to_string(),
                "BYN".to_string(), "EUR".to_string(), "TON".to_string(),
            ],
        };
        Ok(serde_json::to_value(factory_settings)?)
    }
}

impl CurrencyModule {
    async fn get_paged_settings_ui(
        &self,
        owner: &Owner,
        page: usize,
        commander_id: u64,
    ) -> Result<(String, InlineKeyboardMarkup), MyError> {
        let settings: CurrencySettings = Settings::get_module_settings(owner, self.key()).await?;
        let text = format!(
            "⚙️ <b>Настройки модуля</b>: {}\n\nСтатус: {}\n\nВыберите валюты для отображения.",
            self.description(),
            if settings.enabled { "✅ Включен" } else { "❌ Выключен" }
        );

        let toggle_button = InlineKeyboardButton::callback(
            if settings.enabled { "Выключить модуль" } else { "Включить модуль" },
            format!("{}:settings:toggle_module:{}", self.key(), commander_id),
        );

        let all_currencies = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap())?;

        let back_button = InlineKeyboardButton::callback(
            "⬅️ Назад",
            format!("settings_back:{}:{}:{}", owner.r#type, owner.id, commander_id),
        );

        let mut keyboard = Paginator::from(self.key(), &all_currencies)
            .per_page(12)
            .columns(3)
            .current_page(page)
            .add_bottom_row(vec![back_button])
            .set_callback_prefix(format!("{}:settings", self.key()))
            .build(|currency| {
                let is_selected = settings.selected_codes.contains(&currency.code);
                let icon = if is_selected { "✅" } else { "❌" };
                let button_text = format!("{} {}", icon, currency.code);
                let callback_data = format!(
                    "{}:settings:toggle:{}:{}",
                    self.key(),
                    currency.code,
                    commander_id
                );
                InlineKeyboardButton::callback(button_text, callback_data)
            });

        keyboard.inline_keyboard.insert(0, vec![toggle_button]);

        Ok((text, keyboard))
    }
}

pub async fn currency_codes_handler(bot: Bot, msg: Message, code: String) -> Result<(), MyError> {
    if msg.chat.is_private() {
        handle_currency_update::<User>(bot, msg, code).await
    } else {
        handle_currency_update::<Group>(bot, msg, code).await
    }
}