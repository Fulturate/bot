use crate::config::Config;
use crate::util::errors::MyError;
use crate::util::transcription::{
    back_handler, delete_transcription_handler, summarization_handler,
};
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;

pub(crate) async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let _config = Config::new().await;

    tokio::spawn(async move {
        let qq = q.clone();
        let data = qq.data.clone().unwrap();

        if data.starts_with("delete_") {
            delete_transcription_handler(bot, qq).await
        } else if data.starts_with("summarize") {
            summarization_handler(bot, qq, &_config).await
        } else if data.starts_with("back_to_full") {
            back_handler(bot, qq, &_config).await
        } else {
            bot.answer_callback_query(qq.id).await.unwrap();
            Ok(())
        }
    });

    Ok(())
}
