use crate::{
    bot::{
        callbacks::{
            cobalt_pagination::handle_cobalt_pagination,
            delete::{
                handle_delete_confirmation, handle_delete_data_confirmation,
                handle_delete_request,
            },
            translate::handle_translate_callback,
            whisper::handle_whisper_callback,
        },
        commands::settings::update_settings_message,
        modules::{registry::MOD_MANAGER, Owner},
    },
    core::{
        config::Config,
        services::speech_recognition::{back_handler, pagination_handler, summarization_handler},
    },
    errors::MyError,
};
use std::sync::Arc;
use teloxide::{
    payloads::EditMessageTextSetters,
    prelude::{CallbackQuery, Requester},
    Bot,
};
use crate::bot::callbacks::delete::handle_delete_data;

pub mod cobalt_pagination;
pub mod delete;
pub mod translate;
pub mod whisper;

enum CallbackAction<'a> {
    ModuleSettings {
        module_key: &'a str,
        rest: &'a str,
    },
    ModuleSelect {
        owner_type: &'a str,
        owner_id: &'a str,
        module_key: &'a str,
    },
    SettingsBack {
        owner_type: &'a str,
        owner_id: &'a str,
    },
    CobaltPagination,
    DeleteData,
    DeleteDataConfirmation,
    DeleteMessage,
    DeleteConfirmation,
    Summarize,
    SpeechPage,
    BackToFull,
    Whisper,
    Translate,
    NoOp,
}

fn parse_callback_data(data: &'_ str) -> Option<CallbackAction<'_>> {
    if data == "noop" {
        return Some(CallbackAction::NoOp);
    }

    if let Some(rest) = data.strip_prefix("module_select:") {
        let parts: Vec<_> = rest.split(':').collect();
        if parts.len() == 3 {
            return Some(CallbackAction::ModuleSelect {
                owner_type: parts[0],
                owner_id: parts[1],
                module_key: parts[2],
            });
        }
    }

    if let Some(rest) = data.strip_prefix("settings_back:") {
        let parts: Vec<_> = rest.split(':').collect();
        if parts.len() == 2 {
            return Some(CallbackAction::SettingsBack {
                owner_type: parts[0],
                owner_id: parts[1],
            });
        }
    }

    if let Some(module_key) = MOD_MANAGER.get_all_modules().iter().find_map(|m| {
        data.starts_with(&format!("{}:settings:", m.key()))
            .then_some(m.key())
    }) {
        let rest = data
            .strip_prefix(&format!("{}:settings:", module_key))
            .unwrap_or("");
        return Some(CallbackAction::ModuleSettings { module_key, rest });
    }

    if data.starts_with("delete_data_confirm:") {
        return Some(CallbackAction::DeleteDataConfirmation);
    }
    if data == "delete_data" {
        return Some(CallbackAction::DeleteData);
    }
    if data.starts_with("delete_msg") {
        return Some(CallbackAction::DeleteMessage);
    }
    if data.starts_with("delete_confirm:") {
        return Some(CallbackAction::DeleteConfirmation);
    }
    if data.starts_with("summarize") {
        return Some(CallbackAction::Summarize);
    }
    if data.starts_with("speech:page:") {
        return Some(CallbackAction::SpeechPage);
    }
    if data.starts_with("back_to_full") {
        return Some(CallbackAction::BackToFull);
    }
    if data.starts_with("whisper") {
        return Some(CallbackAction::Whisper);
    }
    if data.starts_with("tr_") || data.starts_with("tr:") {
        return Some(CallbackAction::Translate);
    }
    if data.starts_with("cobalt:") {
        return Some(CallbackAction::CobaltPagination);
    }

    None
}

pub async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let config = Arc::new(Config::new().await);

    let Some(data) = &q.data else {
        return Ok(());
    };

    match parse_callback_data(data) {
        Some(CallbackAction::ModuleSelect {
                 owner_type,
                 owner_id,
                 module_key,
             }) => {
            if let (Some(module), Some(message)) = (MOD_MANAGER.get_module(module_key), &q.message)
            {
                let owner = Owner {
                    id: owner_id.to_string(),
                    r#type: owner_type.to_string(),
                };
                let (text, keyboard) = module.get_settings_ui(&owner).await?;
                bot.edit_message_text(message.chat().id, message.id(), text)
                    .reply_markup(keyboard)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            }
        }
        Some(CallbackAction::SettingsBack {
                 owner_type,
                 owner_id,
             }) => {
            if let Some(message) = q.message {
                update_settings_message(bot, message, owner_id.to_string(), owner_type.to_string())
                    .await?;
            }
        }
        Some(CallbackAction::ModuleSettings { module_key, rest }) => {
            if let (Some(module), Some(message)) = (MOD_MANAGER.get_module(module_key), &q.message)
            {
                let owner = Owner {
                    id: message.chat().id.to_string(),
                    r#type: (if message.chat().is_private() {
                        "user"
                    } else {
                        "group"
                    })
                        .to_string(),
                };
                module.handle_callback(bot, &q, &owner, rest).await?;
            }
        }
        Some(CallbackAction::CobaltPagination) => handle_cobalt_pagination(bot, q, config).await?,
        Some(CallbackAction::DeleteData) => handle_delete_data(bot, q).await?,
        Some(CallbackAction::DeleteDataConfirmation) => {
            handle_delete_data_confirmation(bot, q).await?
        }
        Some(CallbackAction::DeleteMessage) => handle_delete_request(bot, q).await?,
        Some(CallbackAction::DeleteConfirmation) => {
            handle_delete_confirmation(bot, q, &config).await?
        }
        Some(CallbackAction::Summarize) => summarization_handler(bot, q, &config).await?,
        Some(CallbackAction::SpeechPage) => pagination_handler(bot, q, &config).await?,
        Some(CallbackAction::BackToFull) => back_handler(bot, q, &config).await?,
        Some(CallbackAction::Whisper) => handle_whisper_callback(bot, q, &config).await?,
        Some(CallbackAction::Translate) => handle_translate_callback(bot, q, &config).await?,
        Some(CallbackAction::NoOp) => {
            bot.answer_callback_query(q.id).await?;
        }
        None => {
            log::warn!("Unhandled callback query data: {}", data);
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}