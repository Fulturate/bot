use crate::{
    bot::{
        callbacks::{
            cobalt_pagination::handle_cobalt_pagination,
            delete::{
                handle_delete_confirmation, handle_delete_data, handle_delete_data_confirmation,
                handle_delete_request,
            },
            translate::handle_translate_callback,
            whisper::handle_whisper_callback,
        },
        commands::settings::update_settings_message,
        modules::{Owner, registry::MOD_MANAGER},
    },
    core::{
        config::Config,
        services::speech_recognition::{back_handler, pagination_handler, summarization_handler},
    },
    errors::MyError,
};
use log::info;
use std::sync::Arc;
use teloxide::{
    Bot,
    payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters},
    prelude::{CallbackQuery, Requester},
};
use crate::core::services::speech_recognition::retry_speech_handler;

pub mod cobalt_pagination;
pub mod delete;
pub mod translate;
pub mod whisper;

enum CallbackAction<'a> {
    ModuleSettings {
        module_key: &'a str,
        rest: &'a str,
        commander_id: u64,
    },
    ModuleSelect {
        owner_type: &'a str,
        owner_id: &'a str,
        module_key: &'a str,
        commander_id: u64,
    },
    SettingsBack {
        owner_type: &'a str,
        owner_id: &'a str,
        commander_id: u64,
    },
    DeleteData {
        commander_id: u64,
    },
    CobaltPagination,
    DeleteDataConfirmation,
    DeleteMessage,
    DeleteConfirmation,
    Summarize {
        user_id: u64,
    },
    RetrySpeech {
        message_id: i32,
        user_id: u64,
        action_type: &'a str, // "transcribe" or "summarize"
        attempt: u32,
    },
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
        if parts.len() == 4
            && let Ok(commander_id) = parts[3].parse()
        {
            return Some(CallbackAction::ModuleSelect {
                owner_type: parts[0],
                owner_id: parts[1],
                module_key: parts[2],
                commander_id,
            });
        }
    }

    if let Some(rest) = data.strip_prefix("settings_back:") {
        let parts: Vec<_> = rest.split(':').collect();
        if parts.len() == 3
            && let Ok(commander_id) = parts[2].parse()
        {
            return Some(CallbackAction::SettingsBack {
                owner_type: parts[0],
                owner_id: parts[1],
                commander_id,
            });
        }
    }

    if let Some(module_key) = MOD_MANAGER.get_all_modules().iter().find_map(|m| {
        data.starts_with(&format!("{}:settings:", m.key()))
            .then_some(m.key())
    }) {
        let full_rest = data
            .strip_prefix(&format!("{}:settings:", module_key))
            .unwrap_or_default();

        if let Some((rest, id_str)) = full_rest.rsplit_once(':')
            && let Ok(commander_id) = id_str.parse()
        {
            return Some(CallbackAction::ModuleSettings {
                module_key,
                rest,
                commander_id,
            });
        }

        return Some(CallbackAction::ModuleSettings {
            module_key,
            rest: full_rest,
            commander_id: 0,
        });
    }

    if let Some(commander_id_str) = data.strip_prefix("delete_data:")
        && let Ok(commander_id) = commander_id_str.parse()
    {
        return Some(CallbackAction::DeleteData { commander_id });
    }

    if data.starts_with("delete_data_confirm:") {
        return Some(CallbackAction::DeleteDataConfirmation);
    }
    if data.starts_with("delete_msg") {
        return Some(CallbackAction::DeleteMessage);
    }
    if data.starts_with("delete_confirm:") {
        return Some(CallbackAction::DeleteConfirmation);
    }
    // if data.starts_with("summarize") {
    if let Some(author_id) = data.strip_prefix("summarize:")
        && let Ok(author_id) = author_id.parse()
    {
        return Some(CallbackAction::Summarize { user_id: author_id });
    }
    if let Some(rest) = data.strip_prefix("retry_speech:") {
        let parts: Vec<_> = rest.splitn(4, ':').collect();
        if parts.len() == 4
            && let Ok(message_id) = parts[0].parse()
            && let Ok(user_id) = parts[1].parse()
            && let Ok(attempt) = parts[3].parse()
        {
            return Some(CallbackAction::RetrySpeech {
                message_id,
                user_id,
                action_type: parts[2], // ahh tupoy The trait bound `&str: FromStr` is not satisfied
                attempt
            });
        }
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

    let Some(data) = &q.data.clone() else {
        return Ok(());
    };

    match parse_callback_data(data) {
        Some(CallbackAction::ModuleSelect {
            owner_type,
            owner_id,
            module_key,
            commander_id,
        }) => {
            info!(
                "module_select: id: {} | commander_id: {}",
                q.from.clone().id.0,
                commander_id
            );
            if q.from.id.0 != commander_id {
                bot.answer_callback_query(q.id)
                    .text("❌ Вы не можете управлять этими настройками.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
            if let (Some(module), Some(message)) = (MOD_MANAGER.get_module(module_key), &q.message)
            {
                let owner = Owner {
                    id: owner_id.to_string(),
                    r#type: owner_type.to_string(),
                };
                let (text, keyboard) = module.get_settings_ui(&owner, commander_id).await?;
                bot.edit_message_text(message.chat().id, message.id(), text)
                    .reply_markup(keyboard)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;
            }
        }
        Some(CallbackAction::SettingsBack {
            owner_type,
            owner_id,
            commander_id,
        }) => {
            info!(
                "settings_back: id: {} | commander_id: {}",
                q.from.clone().id.0,
                commander_id
            );
            if q.from.id.0 != commander_id {
                bot.answer_callback_query(q.id)
                    .text("❌ Вы не можете управлять этими настройками.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
            if let Some(message) = q.message {
                update_settings_message(
                    bot,
                    message,
                    owner_id.to_string(),
                    owner_type.to_string(),
                    commander_id,
                )
                .await?;
            }
        }
        Some(CallbackAction::ModuleSettings {
            module_key,
            rest,
            commander_id,
        }) => {
            info!(
                "module_settings: id: {} | commander_id: {}",
                q.from.clone().id.0,
                commander_id
            );
            if q.from.id.0 != commander_id {
                bot.answer_callback_query(q.id)
                    .text("❌ Вы не можете управлять этими настройками.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
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
                module
                    .handle_callback(bot, &q, &owner, rest, commander_id)
                    .await?;
            }
        }
        Some(CallbackAction::DeleteData { commander_id }) => {
            if q.from.id.0 != commander_id {
                bot.answer_callback_query(q.id)
                    .text("❌ Вы не можете управлять этими настройками.")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
            handle_delete_data(bot, q).await?
        }
        Some(CallbackAction::CobaltPagination) => handle_cobalt_pagination(bot, q, config).await?,
        Some(CallbackAction::DeleteDataConfirmation) => {
            handle_delete_data_confirmation(bot, q).await?
        }
        Some(CallbackAction::DeleteMessage) => handle_delete_request(bot, q).await?,
        Some(CallbackAction::DeleteConfirmation) => {
            handle_delete_confirmation(bot, q, &config).await?
        }
        Some(CallbackAction::Summarize {user_id}) => summarization_handler(bot, q, &config, user_id).await?,
        Some(CallbackAction::RetrySpeech { message_id, user_id, action_type, attempt }) => {
            retry_speech_handler(bot, q.clone(), &config, message_id, user_id, action_type, attempt).await?
        }
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