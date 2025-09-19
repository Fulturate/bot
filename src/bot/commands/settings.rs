use crate::bot::modules::Owner;
use crate::bot::modules::registry::MOD_MANAGER;
use crate::core::db::schemas::settings::Settings;
use crate::errors::MyError;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage};

pub async fn settings_command_handler(bot: Bot, message: Message) -> Result<(), MyError> {
    let owner_id = message.chat.id.to_string();
    let owner_type = if message.chat.is_private() {
        "user"
    } else {
        "group"
    }
    .to_string();

    let settings_doc = Settings::create_with_defaults(&Owner {
        id: owner_id.clone(),
        r#type: owner_type.clone(),
    })
    .await?;

    let kb_buttons: Vec<Vec<InlineKeyboardButton>> = MOD_MANAGER
        .get_all_modules()
        .into_iter()
        .map(|module| {
            let settings: serde_json::Value = settings_doc
                .modules
                .get(module.key())
                .cloned()
                .unwrap_or_default();

            let is_enabled = settings
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let status = if is_enabled { "✅" } else { "❌" };
            let text = format!("{} {}", status, module.description());
            let callback_data =
                format!("module_select:{}:{}:{}", owner_type, owner_id, module.key());

            vec![InlineKeyboardButton::callback(text, callback_data)]
        })
        .collect();

    let keyboard = InlineKeyboardMarkup::new(kb_buttons);

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
    let settings_doc = Settings::get_or_create(&Owner {
        id: owner_id.clone(),
        r#type: owner_type.clone(),
    })
    .await?;

    let kb_buttons: Vec<Vec<InlineKeyboardButton>> = MOD_MANAGER
        .get_all_modules()
        .into_iter()
        .map(|module| {
            let settings: serde_json::Value = settings_doc
                .modules
                .get(module.key())
                .cloned()
                .unwrap_or_default();
            let is_enabled = settings
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let status = if is_enabled { "✅" } else { "❌" };
            let text = format!("{} {}", status, module.description());
            let callback_data =
                format!("module_select:{}:{}:{}", owner_type, owner_id, module.key());

            vec![InlineKeyboardButton::callback(text, callback_data)]
        })
        .collect();

    let keyboard = InlineKeyboardMarkup::new(kb_buttons);

    let text = "Настройки модулей:";

    if let MaybeInaccessibleMessage::Regular(msg) = message {
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, text)
            .reply_markup(keyboard)
            .await;
    }

    Ok(())
}
