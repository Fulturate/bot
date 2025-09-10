use crate::bot::inlines::whisper::Whisper;
use crate::core::config::Config;
use crate::errors::MyError;
use teloxide::Bot;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters};
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::InlineKeyboardMarkup;

pub async fn handle_whisper_callback(
    bot: Bot,
    q: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let data = q.data.as_ref().ok_or("Callback query data is empty")?;

    let parts: Vec<&str> = data.split('_').collect();
    if parts.len() != 3 || parts[0] != "whisper" {
        return Ok(());
    }
    let action = parts[1];
    let whisper_id = parts[2];

    let user = q.from.clone();
    let _username = user.username.clone().unwrap_or_default();

    let redis_key = format!("whisper:{}", whisper_id);

    let whisper: Option<Whisper> = config.get_redis_client().get(&redis_key).await?;

    let whisper = match whisper {
        Some(w) => w,
        None => {
            bot.answer_callback_query(q.id)
                .text("❌ Этот шепот истек или был забыт.")
                .show_alert(true)
                .await?;
            return Ok(());
        }
    };

    let is_sender = user.id.0 == whisper.sender_id;
    let is_recipient = whisper.recipients.iter().any(|r| {
        if r.id == Some(user.id.0) {
            return true;
        }

        if let (Some(recipient_username), Some(username)) = (
            &r.username,
            &user.username.as_ref().map(|s| s.to_lowercase()),
        ) && recipient_username == username
        {
            return true;
        }
        false
    });

    if !is_sender && !is_recipient {
        bot.answer_callback_query(q.id)
            .text("🤫 Это не для тебя.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    match action {
        "read" => {
            let alert_text = format!("{}", whisper.content);
            bot.answer_callback_query(q.id)
                .text(alert_text)
                .show_alert(true)
                .await?;
        }
        "forget" => {
            config.get_redis_client().delete(&redis_key).await?;
            bot.answer_callback_query(q.id).text("Шепот забыт.").await?;

            if let Some(message) = q.message {
                bot.edit_message_text(
                    message.chat().id,
                    message.id(),
                    format!("🤫 Шепот от {} был забыт.", whisper.sender_first_name),
                )
                .reply_markup(InlineKeyboardMarkup::new(vec![vec![]]))
                .await?;
            }
        }
        _ => {}
    }

    Ok(())
}
