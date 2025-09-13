use crate::bot::callbacks::cobalt_pagination::handle_cobalt_pagination;
use crate::bot::callbacks::delete::{handle_delete_confirmation, handle_delete_request};
use crate::bot::callbacks::module::{
    module_option_handler, module_select_handler, module_toggle_handler, settings_back_handler,
    settings_set_handler,
};
use crate::bot::callbacks::transcription::{
    back_handler, pagination_handler, summarization_handler,
};
use crate::bot::callbacks::translate::handle_translate_callback;
use crate::bot::callbacks::whisper::handle_whisper_callback;
use crate::core::config::Config;
use crate::errors::MyError;
use std::sync::Arc;
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::Bot;

pub mod cobalt_pagination;
pub mod delete;
pub mod module;
pub mod transcription;
pub mod translate;
pub mod whisper;

pub async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let config = Arc::new(Config::new().await);

    if let Some(data) = &q.data {
        if data.starts_with("settings_set:") {
            settings_set_handler(bot, q).await?
        } else if data.starts_with("delete_msg:") {
            handle_delete_request(bot, q).await?
        } else if data.starts_with("delete_confirm:") {
            handle_delete_confirmation(bot, q, &config).await?
        } else if data.starts_with("summarize") {
            summarization_handler(bot, q, &config).await?
        } else if data.starts_with("back_to_full") {
            back_handler(bot, q, &config).await?
        } else if data.starts_with("paginate:") {
            pagination_handler(bot, q, &config).await?
        } else if data.starts_with("module_select:") {
            module_select_handler(bot, q).await?
        } else if data.starts_with("module_toggle") {
            module_toggle_handler(bot, q).await?
        } else if data.starts_with("module_opt:") {
            module_option_handler(bot, q).await?
        } else if data.starts_with("settings_back:") {
            settings_back_handler(bot, q).await?
        } else if data.starts_with("whisper") {
            handle_whisper_callback(bot, q, &config).await?
        } else if data.starts_with("tr_") {
            handle_translate_callback(bot, q, &config).await?
        } else if data.starts_with("cobalt:") {
            handle_cobalt_pagination(bot, q, config).await?
        } else {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}