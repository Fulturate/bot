use crate::config::Config;
use crate::handlers::markups::inlines::delete_msg_inline::delete_msg_handler;
use crate::util::errors::MyError;
use teloxide::requests::Requester;
use teloxide::types::CallbackQuery;
use teloxide::Bot;

pub(crate) async fn callback_query_handlers(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let config = Config::new().await;

    tokio::spawn(async move {
        let qq = q.clone();
        let data = qq.data.clone().unwrap();

        if data.starts_with("delete_msg") {
            delete_msg_handler(bot, qq, &config).await
        } else {
            bot.answer_callback_query(qq.id).await.unwrap();
            Ok(())
        }
    });

    Ok(())
}
