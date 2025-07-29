use crate::util::errors::MyError;
use crate::util::inline::DELETE_CALLBACK_DATA;
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::CallbackQuery;
use teloxide::requests::Requester;

pub async fn delete_msg_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    if let Some(data) = q.data {
        let parts: Vec<&str> = data.split(':').collect();

        if parts.len() == 2 && parts[0] == DELETE_CALLBACK_DATA {
            if let Ok(original_user_id) = parts[1].parse::<u64>() {
                if let Some(message) = q.message {
                    let chat_id = message.chat().id;
                    let member = bot.get_chat_member(chat_id, q.from.id).await;

                    if let Ok(member) = member {
                        if member.user.id.0.eq(&original_user_id) || member.is_privileged() {
                            bot.delete_message(chat_id, message.id()).await?;
                            return Ok(());
                        }
                    }
                }

                bot.answer_callback_query(q.id)
                    .text("❌ Произошла неопознанная ошибка")
                    .show_alert(true)
                    .await?;
                return Ok(());
            }
        }
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}
