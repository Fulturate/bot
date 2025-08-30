use super::super::inlines::whisper::Whisper;
use crate::config::Config;
use crate::util::errors::MyError;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters};
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::InlineKeyboardMarkup;
use teloxide::Bot;

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
    let username = user.username.clone().unwrap_or_default();

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
        if r.id != 0 {
            r.id == user.id.0
        } else if let Some(uname) = &r.username {
            uname == &username
        } else {
            false
        }
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
            bot.answer_callback_query(q.id)
                .text("Шепот забыт.")
                .await?;

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