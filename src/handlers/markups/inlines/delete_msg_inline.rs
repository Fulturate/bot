use crate::util::errors::MyError;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::CallbackQuery;
use teloxide::requests::Requester;
use teloxide::Bot;

pub async fn delete_msg_handler(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), MyError> {
    if let Some(data) = q.data {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() == 2 && parts[0] == crate::util::inline::DELETE_CALLBACK_DATA {
            if let Ok(original_user_id) = parts[1].parse::<u64>() {
                if q.from.id.0.eq(&original_user_id) {
                    if let Some(message) = q.message {
                        bot.delete_message(message.chat().id, message.id()).await?;
                    }
                } else {
                    bot.answer_callback_query(q.id)
                        .text("❌ Вы не можете удалить чужое сообщение")
                        .show_alert(true)
                        .await?;
                    return Ok(());
                }
            }
        }
    }

    bot.answer_callback_query(q.id).await?;
    Ok(())
}
