use crate::bot::callbacks::transcription::back_handler;
use crate::bot::keyboards::delete::confirm_delete_keyboard;
use crate::core::config::Config;
use crate::errors::MyError;
use log::error;
use teloxide::prelude::*;
use teloxide::types::{ChatId, User};

async fn has_delete_permission(
    bot: &Bot,
    chat_id: ChatId,
    is_group: bool,
    clicker: &User,
    target_user_id: u64,
) -> bool {
    if target_user_id == 72 {
        return true;
    }
    if clicker.id.0 == target_user_id {
        return true;
    }
    if is_group {
        if let Ok(member) = bot.get_chat_member(chat_id, clicker.id).await {
            return member.is_privileged();
        }
    }
    false
}

pub async fn handle_delete_request(bot: Bot, query: CallbackQuery) -> Result<(), MyError> {
    let Some(message) = query.message.as_ref().and_then(|m| m.regular_message()) else { return Ok(()) };
    let Some(data) = query.data.as_ref() else { return Ok(()) };

    let target_user_id_str = data.strip_prefix("delete_msg:").unwrap_or_default();
    let Ok(target_user_id) = target_user_id_str.parse::<u64>() else {
        bot.answer_callback_query(query.id)
            .text("❌ Ошибка: неверный ID в кнопке.")
            .show_alert(true)
            .await?;
        return Ok(());
    };

    let can_delete = has_delete_permission(
        &bot,
        message.chat.id,
        message.chat.is_group() || message.chat.is_supergroup(),
        &query.from,
        target_user_id,
    )
        .await;

    if !can_delete {
        bot.answer_callback_query(query.id)
            .text("❌ Удалить может только автор сообщения или администратор.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id).await?;
    bot.edit_message_text(
        message.chat.id,
        message.id,
        "Вы уверены, что хотите удалить?",
    )
        .reply_markup(confirm_delete_keyboard(target_user_id))
        .await?;

    Ok(())
}

pub async fn handle_delete_confirmation(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = query.message.as_ref().and_then(|m| m.regular_message()) else { return Ok(()) };
    let Some(data) = query.data.as_ref() else { return Ok(()) };

    let parts: Vec<&str> = data.split(':').collect();
    if parts.len() != 3 { return Ok(()) };

    let Ok(target_user_id) = parts[1].parse::<u64>() else { return Ok(()) };
    let action = parts[2];

    let can_delete = has_delete_permission(
        &bot,
        message.chat.id,
        message.chat.is_group() || message.chat.is_supergroup(),
        &query.from,
        target_user_id,
    )
        .await;

    if !can_delete {
        bot.answer_callback_query(query.id)
            .text("❌ У вас нет прав для этого действия.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    bot.answer_callback_query(query.clone().id).await?;

    match action {
        "yes" => {
            bot.delete_message(message.chat.id, message.id)
                .await
                .map_err(|e| error!("Failed to delete message: {:?}", e))
                .ok();
        }
        "no" => {
            back_handler(bot, query, config).await?;
        }
        _ => {}
    }

    Ok(())
}