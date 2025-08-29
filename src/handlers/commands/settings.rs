use crate::config::Config;
use crate::db::schemas::settings::Settings;
use crate::db::schemas::{BaseFunctions, CurrenciesFunctions, SettingsRepo};
use crate::{
    db::functions::get_or_create,
    db::schemas::group::Group,
    db::schemas::user::User,
    util::{
        currency::converter::{get_all_currency_codes, CURRENCY_CONFIG_PATH},
        errors::MyError,
    },
};
use log::error;
use oximod::Model;
use std::collections::HashSet;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage, ParseMode, ReplyParameters};

pub async fn handle_currency_update<T: BaseFunctions + CurrenciesFunctions + Model>(
    bot: Bot,
    msg: Message,
    code: String,
) -> Result<(), MyError> {
    let code = code.to_uppercase();

    let all_codes = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap())?;
    if !all_codes.iter().any(|c| c.code == code) {
        let message = format!("Currency code <code>{}</code> does not exist.", code);
        bot.send_message(msg.chat.id, message)
            .parse_mode(ParseMode::Html)
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }

    let entity_id = msg.chat.id.to_string();
    let entity = match get_or_create::<T>(entity_id).await {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to get or create entity: {:?}", e);
            let message = "Error: Could not access settings. Try again".to_string();
            bot.send_message(msg.chat.id, message)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            return Ok(());
        }
    };

    let is_enabled = entity.get_currencies().iter().any(|c| c.code == code);

    let (update_func, action_text) = if is_enabled {
        (T::remove_currency(entity.get_id(), &code), "removed")
    } else {
        let currency_to_add = all_codes.iter().find(|x| x.code == code).unwrap();
        (T::add_currency(entity.get_id(), currency_to_add), "added")
    };

    let message = match update_func.await {
        Ok(_) => {
            format!(
                "Successfully {} <code>{}</code> from currency conversion.",
                action_text, code
            )
        }
        Err(e) => {
            error!("--- Update failed: {:?} ---", e);
            "Failed to apply changes.".to_string()
        }
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}

pub async fn currency_codes_handler(bot: Bot, msg: Message, code: String) -> Result<(), MyError> {
    if msg.chat.is_private() {
        handle_currency_update::<User>(bot, msg, code).await
    } else {
        handle_currency_update::<Group>(bot, msg, code).await
    }
}

async fn get_enabled_codes(msg: &Message) -> HashSet<String> {
    let chat_id = msg.chat.id.to_string();
    if msg.chat.is_private() {
        if let Ok(Some(user)) = <User as BaseFunctions>::find_by_id(chat_id).await {
            return user
                .get_currencies()
                .iter()
                .map(|c| c.code.clone())
                .collect();
        }
    } else if let Ok(Some(group)) = <Group as BaseFunctions>::find_by_id(chat_id).await {
        return group
            .get_currencies()
            .iter()
            .map(|c| c.code.clone())
            .collect();
    }

    HashSet::new()
}

pub async fn currency_codes_list_handler(bot: Bot, msg: Message) -> Result<(), MyError> {
    let mut message = String::from("Could not load available currencies.");

    if let Ok(codes) = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap()) {
        let enabled_codes = get_enabled_codes(&msg).await;

        let codes_list = codes
            .iter()
            .map(|currency| {
                let icon = if enabled_codes.contains(&currency.code) {
                    "✅"
                } else {
                    "❌"
                };
                format!("{} {} - {}", currency.flag, currency.code, icon)
            })
            .collect::<Vec<String>>()
            .join("\n");

        // safety: assume that bot commands always exists
        let result = bot
            .get_my_commands()
            .await
            .unwrap()
            .iter()
            .find_map(|command| (&command.command == "setcurrency").then(|| command.clone()));

        if let Some(command) = result {
            message = format!(
                // FIXME: better hardcoding <3
                "Available currencies to set up: <blockquote expandable>{}</blockquote>\n\nUsage: <code>/{} CURRENCY_CODE</code> (e.g., <code>/{} UAH</code>) to enable/disable it.\n\nNotes:\n✅ - enabled\n❌ - disabled",
                codes_list, &command.command, &command.command
            );
        } else {
            // default fallback
            message = format!(
                "Available currencies to set up: <blockquote expandable>{}</blockquote>\n\nNotes:\n✅ - enabled\n❌ - disabled",
                codes_list
            );
        }
    }

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}


// new settings
pub async fn settings_command_handler(
    bot: Bot,
    message: Message,
    _config: &Config,
) -> Result<(), MyError> {
    let owner_id: String = if let Some(user) = message.from {
        user.id.to_string()
    } else {
        message.chat.id.to_string()
    };

    let owner_type = if message.chat.is_private() { "user" } else { "group" };

    let settings = Settings::get_or_create(&owner_id, &owner_type).await?;

    let keyboard = InlineKeyboardMarkup::new(
        settings
            .modules
            .iter()
            .map(|m| {
                let status = if m.enabled { "✅" } else { "❌" };
                let text = format!("{status} {}", m.description);
                let callback_data = format!("module_select:{owner_type}:{owner_id}:{}", m.key);
                vec![InlineKeyboardButton::callback(text, callback_data)]
            })
            .collect::<Vec<_>>(),
    );

    bot.send_message(message.chat.id, "Настройки модулей:")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn update_settings_message(
    bot: Bot,
    message: MaybeInaccessibleMessage,
    owner_id: String,
    owner_type: String,
) -> Result<(), MyError> {
    let settings = Settings::get_or_create(&owner_id, &owner_type).await?;

    let keyboard = InlineKeyboardMarkup::new(
        settings
            .modules
            .iter()
            .map(|m| {
                let status = if m.enabled { "✅" } else { "❌" };
                let text = format!("{status} {}", m.description);
                let callback_data = format!("module_select:{owner_type}:{owner_id}:{}", m.key);
                vec![InlineKeyboardButton::callback(text, callback_data)]
            })
            .collect::<Vec<_>>(),
    );

    let text = "Настройки модулей:";

    if let MaybeInaccessibleMessage::Regular(msg) = message {
        let _ = bot.edit_message_text(msg.chat.id, msg.id, text)
            .reply_markup(keyboard)
            .await;
    }

    Ok(())
}