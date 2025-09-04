use crate::config::Config;
use crate::handlers::markups::callbacks::cobalter_pagination::handle_cobalt_pagination;
use crate::handlers::markups::callbacks::delete::delete_message_handler;
use crate::handlers::markups::callbacks::module::{
    module_option_handler, module_select_handler, module_toggle_handler, settings_back_handler,
    settings_set_handler,
};
use crate::handlers::markups::callbacks::translate::handle_translate_callback;
use crate::handlers::markups::callbacks::whisper::handle_whisper_callback;
use crate::util::errors::MyError;
use crate::util::transcription::{back_handler, summarization_handler};
use std::sync::Arc;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;
use teloxide::Bot;

pub(crate) async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let config = Arc::new(Config::new().await);

    if let Some(data) = &q.data {
        if data.starts_with("settings_set:") {
            settings_set_handler(bot, q).await?
        } else if data.starts_with("delete_msg") {
            delete_message_handler(bot, q).await?
        } else if data.starts_with("summarize") {
            summarization_handler(bot, q, &config).await?
        } else if data.starts_with("back_to_full") {
            back_handler(bot, q, &config).await?
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
        } else if data.starts_with("cobalt_page:") {
            handle_cobalt_pagination(bot, q, config).await?
        } else {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}