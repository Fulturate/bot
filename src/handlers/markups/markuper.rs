use crate::config::Config;
use crate::util::errors::MyError;
use crate::util::transcription::{
    back_handler, summarization_handler,
};
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;
use crate::handlers::markups::callbacks::delete::delete_message_handler;
use crate::handlers::markups::callbacks::module::{module_option_handler, module_select_handler, module_toggle_handler, settings_back_handler};
use crate::handlers::markups::callbacks::translate::handle_translate_callback;
use crate::handlers::markups::callbacks::whisper::handle_whisper_callback;

pub(crate) async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let _config = Config::new().await;

    // tokio::spawn(async move {
        let qq = q.clone();
        let data = qq.data.clone().unwrap();

        if data.starts_with("delete_msg") {
            delete_message_handler(bot, qq).await?
        } else if data.starts_with("summarize") {
            summarization_handler(bot, qq, &_config).await?
        } else if data.starts_with("back_to_full") {
            back_handler(bot, qq, &_config).await?
        } else if data.starts_with("module_select:") {
            module_select_handler(bot, qq).await?
        } else if data.starts_with("module_toggle") {
            module_toggle_handler(bot, qq).await?
        } else if data.starts_with("module_opt:") {
            module_option_handler(bot, qq).await?
        } else if data.starts_with("settings_back:") {
            settings_back_handler(bot, qq).await?
        } else if data.starts_with("whisper") {
            handle_whisper_callback(bot, qq, &_config).await?
        } else if data.starts_with("tr_") {
            handle_translate_callback(bot, qq, &_config).await?
        } else {
            bot.answer_callback_query(qq.id).await.unwrap();
        }
    // });

    Ok(())
}
