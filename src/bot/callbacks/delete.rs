use crate::errors::MyError;
use log::error;
use teloxide::Bot;
use teloxide::payloads::AnswerCallbackQuerySetters;
use teloxide::prelude::{CallbackQuery, Requester};

pub async fn delete_message_handler(bot: Bot, query: CallbackQuery) -> Result<(), MyError> {
    let Some(message_to_delete) = query.message else {
        return Ok(());
    };

    let Some(message) = message_to_delete.regular_message() else {
        return Ok(());
    };

    let Some(data) = query.data else {
        return Ok(());
    };

    let clicker = query.from;

    let target_user_id_str = data.strip_prefix("delete_msg:").unwrap_or_default();
    let Ok(target_user_id) = target_user_id_str.parse::<u64>() else {
        bot.answer_callback_query(query.id)
            .text("❌ Ошибка: неверный ID в кнопке.")
            .show_alert(true)
            .await?;
        return Ok(());
    };

    let mut has_permission = false;

    if target_user_id == 72 {
        has_permission = true;
    } else if clicker.id.0 == target_user_id {
        has_permission = true;
    }

    if !has_permission && message.chat.is_group() || message.chat.is_supergroup() {
        if let Ok(member) = bot.get_chat_member(message.chat.id, clicker.id).await {
            if member.is_privileged() {
                has_permission = true;
            }
        }
    }

    if !has_permission {
        bot.answer_callback_query(query.id)
            .text("❌ Удалить может только автор сообщения или администратор.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id).await?;

    bot.delete_message(message.chat.id, message.id)
        .await
        .map_err(|e| error!("Failed to delete bot's message: {:?}", e))
        .ok();

    Ok(())
}
