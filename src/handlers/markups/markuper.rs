use crate::config::Config;
use crate::handlers::markups::inlines::delete_msg_inline::delete_msg_handler;
use crate::util::errors::MyError;
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;
use crate::util::transcription::summarization_handler;

pub(crate) async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let _config = Config::new().await;

    tokio::spawn(async move {
        let qq = q.clone();
        let data = qq.data.clone().unwrap();

        if data.starts_with("delete_msg") {
            delete_msg_handler(bot, qq).await
        } else if data.starts_with("summarize") {
            summarization_handler(bot, qq, &_config).await
        } else {
            bot.answer_callback_query(qq.id).await.unwrap();
            Ok(())
        }
    });

    Ok(())
}
