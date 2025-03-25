use crate::config::Config;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use crate::util::errors::MyError;
use crate::util::transcription::{get_file_id, save_file_to_memory, Transcription};

pub async fn voice_handler(bot: Bot, msg: Message, _: &Config) -> Result<(), MyError> {
    if let Some(file_id) = get_file_id(&msg) {
        let file_data = save_file_to_memory(&bot, &file_id).await?;
        let transcription = Transcription { data: file_data };
        let text = transcription.to_text().await;

        bot.send_message(msg.chat.id, format!("Транскрипция: {}", text.unwrap_or_else(|e| format!("Ошибка: {}", e))))
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Не удалось найти голосовое сообщение.")
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}
