use crate::config::Config;
use crate::core::db::schemas::SettingsRepo;
use crate::core::db::schemas::group::Group;
use crate::core::db::schemas::settings::Settings;
use crate::core::db::schemas::user::User;
use crate::core::services::currency::{get_enabled_codes, handle_currency_update};
use crate::errors::MyError;
use crate::util::currency::converter::{CURRENCY_CONFIG_PATH, get_all_currency_codes};
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage, ParseMode,
    ReplyParameters,
};

pub async fn currency_codes_handler(bot: Bot, msg: Message, code: String) -> Result<(), MyError> {
    if msg.chat.is_private() {
        handle_currency_update::<User>(bot, msg, code).await
    } else {
        handle_currency_update::<Group>(bot, msg, code).await
    }
}

/// Deprecated, but still working currency settings
///
/// TODO: move currency settings to /settings in near future
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

///
/// new settings
///
/// TODO: Use traits (interfaces) as a basis for other modules to iterate and show them in the settings menu
/// (or this system, maybe, waiting for entire rewrite)
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

    let owner_type = if message.chat.is_private() {
        "user"
    } else {
        "group"
    };

    let settings = Settings::get_or_create(&owner_id, owner_type).await?;

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
        let _ = bot
            .edit_message_text(msg.chat.id, msg.id, text)
            .reply_markup(keyboard)
            .await;
    }

    Ok(())
}
