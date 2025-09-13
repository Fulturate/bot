use crate::bot::keyboards::transcription::{
    create_summary_keyboard, create_transcription_keyboard, TRANSCRIPTION_MODULE_KEY,
};
use crate::core::config::Config;
use crate::core::services::transcription::{
    save_file_to_memory, split_text, summarize_audio, TranscriptionCache,
};
use crate::errors::MyError;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn pagination_handler(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = query.message.as_ref().and_then(|m| m.regular_message()) else {
        return Ok(());
    };
    let Some(data) = query.data.as_ref() else { return Ok(()) };

    let parts: Vec<&str> = data.split(':').collect();
    if !(parts.len() == 3 && parts[0] == TRANSCRIPTION_MODULE_KEY && parts[1] == "page") {
        return Ok(());
    }

    let Ok(page) = parts[2].parse::<usize>() else { return Ok(()) };

    let cache = config.get_redis_client();
    let message_cache_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_cache_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Кнопка устарела.").await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let Some(cache_entry) = cache.get::<TranscriptionCache>(&file_cache_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось найти текст в кеше.").await?;
        return Ok(());
    };

    let text_parts = split_text(&cache_entry.full_text, 4000);
    if page >= text_parts.len() {
        return Ok(());
    }

    let new_text = format!("<blockquote expandable>{}</blockquote>", text_parts[page]);
    let new_keyboard = create_transcription_keyboard(page, text_parts.len(), query.from.id.0);

    if message.text() != Some(new_text.as_str()) || message.reply_markup() != Some(&new_keyboard) {
        bot.edit_message_text(message.chat.id, message.id, new_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(new_keyboard)
            .await?;
    }

    Ok(())
}

pub async fn back_handler(bot: Bot, query: CallbackQuery, config: &Config) -> Result<(), MyError> {
    let Some(message) = query.message.and_then(|m| m.regular_message().cloned()) else { return Ok(()) };

    let cache = config.get_redis_client();
    let message_cache_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_cache_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось найти исходное сообщение.").await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let Some(cache_entry) = cache.get::<TranscriptionCache>(&file_cache_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось найти текст в кеше.").await?;
        return Ok(());
    };

    let text_parts = split_text(&cache_entry.full_text, 4000);
    let keyboard = create_transcription_keyboard(0, text_parts.len(), query.from.id.0);

    bot.edit_message_text(
        message.chat.id,
        message.id,
        format!("<blockquote expandable>{}</blockquote>", text_parts[0]),
    )
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn summarization_handler(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = query.message.and_then(|m| m.regular_message().cloned()) else { return Ok(()) };

    let cache = config.get_redis_client();
    let message_file_map_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_file_map_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Кнопка устарела.").await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let mut cache_entry = match cache.get::<TranscriptionCache>(&file_cache_key).await? {
        Some(entry) => entry,
        None => {
            bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось найти исходное аудио.").await?;
            return Ok(());
        }
    };

    if let Some(cached_summary) = cache_entry.summary {
        let final_text = format!(
            "Краткое содержание:\n<blockquote expandable>{}</blockquote>",
            cached_summary
        );
        bot.edit_message_text(message.chat.id, message.id, final_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(create_summary_keyboard())
            .await?;
        return Ok(());
    }

    bot.edit_message_text(message.chat.id, message.id, "Составляю краткое содержание...").await?;

    let file_data = save_file_to_memory(&bot, &cache_entry.file_id).await?;
    let new_summary =
        summarize_audio(cache_entry.mime_type.clone(), file_data, config.clone()).await?;

    if new_summary.is_empty() || new_summary.contains("Не удалось получить") {
        bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось составить краткое содержание.").await?;
        return Ok(());
    }

    cache_entry.summary = Some(new_summary.clone());
    cache.set(&file_cache_key, &cache_entry, 86400).await?;

    let final_text = format!(
        "Краткое содержание:\n<blockquote expandable>{}</blockquote>",
        new_summary
    );
    bot.edit_message_text(message.chat.id, message.id, final_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(create_summary_keyboard())
        .await?;

    Ok(())
}