use crate::{
    config::Config,
    handlers::markups::callbacks::{
        cobalt_pagination::handle_cobalt_pagination,
        delete::delete_message_handler,
        module::{
            module_option_handler, module_select_handler, module_toggle_handler,
            settings_back_handler, settings_set_handler,
        },
        translate::handle_translate_callback,
        whisper::handle_whisper_callback,
    },
    util::{
        errors::MyError,
        transcription::{back_handler, summarization_handler},
    },
};
use std::sync::Arc;
use teloxide::{Bot, requests::Requester, types::CallbackQuery};

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
        } else if data.starts_with("cobalt:") {
            handle_cobalt_pagination(bot, q, config).await?
        } else {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}
