use crate::config::Config;
use crate::util::errors::MyError;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::CallbackQuery;
use teloxide::requests::Requester;
use teloxide::Bot;

pub async fn delete_msg_handler(
    bot: Bot,
    q: CallbackQuery,
    _: &Config,
) -> Result<(), MyError> {
    if bot.delete_message(q.chat_id().unwrap(), q.message.unwrap().id()).await.is_ok() {
        bot.answer_callback_query(q.id).await?;
    } else {
        bot.answer_callback_query(q.id).await?;
    }
    Ok(())
}
