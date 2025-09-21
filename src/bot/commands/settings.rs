use crate::bot::modules::Owner;
use crate::bot::modules::registry::MOD_MANAGER;
use crate::core::db::schemas::settings::Settings;
use crate::errors::MyError;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage};

pub async fn settings_command_handler(bot: Bot, message: Message) -> Result<(), MyError> {
    let commander_id = message.from().map(|u| u.id.0).ok_or(MyError::UserNotFound)?;

    let owner_id = message.chat.id.to_string();
    let owner_type = if message.chat.is_private() { "user" } else { "group" }.to_string();

    let settings_doc = Settings::get_or_create(&Owner {
        id: owner_id.clone(),
        r#type: owner_type.clone(),
    })
        .await?;

    let text = String::from(
        "⚙️ <b>Настройки модулей</b>\n\n\
        Нажмите на кнопку, чтобы включить или выключить соответствующий модуль.\n\
        ✅ – модуль включён\n\
        ❌ – модуль выключен\n\n\
        <i>Только тот, кто вызвал это сообщение, может управлять настройками.</i>"
    );

    let mut kb_buttons: Vec<Vec<InlineKeyboardButton>> = MOD_MANAGER
        .get_designed_modules(&*owner_type)
        .into_iter()
        .map(|module| {
            let settings: serde_json::Value = settings_doc
                .modules
                .get(module.key())
                .cloned()
                .unwrap_or_default();

            let is_enabled = settings.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let status = if is_enabled { "✅" } else { "❌" };
            let text = format!("{} — {}", status, module.name());

            let callback_data = format!(
                "module_select:{}:{}:{}:{}",
                owner_type, owner_id, module.key(), commander_id
            );

            vec![InlineKeyboardButton::callback(text, callback_data)]
        })
        .collect();

    kb_buttons.push(vec![InlineKeyboardButton::callback(
        "🗑️ Удалить данные с бота",
        format!("delete_data:{}", commander_id),
    )]);

    let keyboard = InlineKeyboardMarkup::new(kb_buttons);

    bot.send_message(message.chat.id, text)
        .parse_mode(teloxide::types::ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn update_settings_message(
    bot: Bot,
    message: MaybeInaccessibleMessage,
    owner_id: String,
    owner_type: String,
    commander_id: u64,
) -> Result<(), MyError> {
    let settings_doc = Settings::get_or_create(&Owner {
        id: owner_id.clone(),
        r#type: owner_type.clone(),
    })
        .await?;

    let text = String::from(
        "⚙️ <b>Настройки модулей</b>\n\n\
        Нажмите на кнопку, чтобы включить или выключить соответствующий модуль.\n\
        ✅ – модуль включён\n\
        ❌ – модуль выключен\n\n\
        <i>Только тот, кто вызвал это сообщение, может управлять настройками.</i>"
    );

    let mut kb_buttons: Vec<Vec<InlineKeyboardButton>> = MOD_MANAGER
        .get_designed_modules(&*owner_type)
        .into_iter()
        .map(|module| {
            let settings: serde_json::Value = settings_doc
                .modules
                .get(module.key())
                .cloned()
                .unwrap_or_default();
            let is_enabled = settings.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let status = if is_enabled { "✅" } else { "❌" };
            let text = format!("{} — {}", status, module.name());

            let callback_data = format!(
                "module_select:{}:{}:{}:{}",
                owner_type, owner_id, module.key(), commander_id
            );

            vec![InlineKeyboardButton::callback(text, callback_data)]
        })
        .collect();

    kb_buttons.push(vec![InlineKeyboardButton::callback(
        "🗑️ Удалить данные с бота",
        format!("delete_data:{}", commander_id),
    )]);

    let keyboard = InlineKeyboardMarkup::new(kb_buttons);

    if let MaybeInaccessibleMessage::Regular(msg) = message {
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .reply_markup(keyboard)
            .await;
    }

    Ok(())
}