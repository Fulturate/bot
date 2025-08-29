use log::error;
use crate::util::errors::MyError;
use crate::util::inline::DELETE_CALLBACK_DATA;
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::CallbackQuery;
use teloxide::requests::Requester;

pub async fn delete_msg_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let data = match q.data {
        Some(ref data) => data,
        None => return Ok(()),
    };

    let parts: Vec<&str> = data.split(':').collect();

    if parts.len() != 2 || parts[0] != DELETE_CALLBACK_DATA {
        bot.answer_callback_query(q.id)
            .text("❌ Неверный формат данных для удаления.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let original_user_id: u64 = match parts[1].parse() {
        Ok(id) => id,
        Err(_) => {
            bot.answer_callback_query(q.id)
                .text("❌ Произошла ошибка (неверный ID пользователя).")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    let message = match q.message {
        Some(msg) => msg,
        None => {
            bot.answer_callback_query(q.id)
                .text("❌ Произошла неопознанная ошибка (сообщение не найдено).")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    let chat_id = message.chat().id;
    let message_id = message.id();

    let member = bot.get_chat_member(chat_id, q.from.id).await;

    if let Ok(member) = member
        && (member.user.id.0 == original_user_id || member.is_privileged())
    {
        return match bot.delete_message(chat_id, message_id).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to delete message: {:?}", e);
                bot.answer_callback_query(q.id)
                        .text("❌ Не удалось удалить сообщение (возможно, у меня нет прав или сообщение слишком старое).")
                        .show_alert(true)
                        .await?;
                Ok(())
            }
        };
    }

    bot.answer_callback_query(q.id)
        .text("❌ У вас нет прав для удаления этого сообщения.")
        .show_alert(true)
        .await?;

    Ok(())
}
