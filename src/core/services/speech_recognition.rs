use crate::{
    bot::keyboards::transcription::{
        create_summary_keyboard, create_transcription_keyboard, create_retry_keyboard,
    },
    core::config::Config,
    errors::MyError,
    util::{enums::AudioStruct, is_admin_or_author, split_text},
};
use bytes::Bytes;
use gem_rs::{
    api::Models,
    client::GemSession,
    types::{Blob, Context, HarmBlockThreshold, Role, Settings},
};
use log::{debug, error, info};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use teloxide::{
    prelude::*,
    types::{FileId, MessageKind, ParseMode, ReplyParameters},
};
use teloxide::utils::html;

#[derive(Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs, Clone)]
pub struct TranscriptionCache {
    pub full_text: String,
    pub summary: Option<String>,
    pub file_id: String,
    pub mime_type: String,
    pub attempt: u32,
}

pub struct Transcription {
    pub(crate) mime_type: String,
    pub(crate) data: Bytes,
    pub(crate) config: Config,
}

pub async fn pagination_handler(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = query.message.as_ref().and_then(|m| m.regular_message()) else {
        return Ok(());
    };
    let Some(data) = query.data.as_ref() else {
        return Ok(());
    };

    let parts: Vec<&str> = data.split(':').collect();
    if !(parts.len() == 3 && parts[0] == "speech" && parts[1] == "page") {
        return Ok(());
    }

    let Ok(page) = parts[2].parse::<usize>() else {
        return Ok(());
    };

    let cache = config.get_redis_client();
    let message_cache_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id): Option<String> = cache.get::<String>(&message_cache_key).await?
    else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Кнопка устарела.")
            .await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let Some(cache_entry) = cache.get::<TranscriptionCache>(&file_cache_key).await? else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось найти текст в кеше.",
        )
            .await?;
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
    let Some(message) = query.message.and_then(|m| m.regular_message().cloned()) else {
        return Ok(());
    };

    let cache = config.get_redis_client();
    let message_cache_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_cache_key).await? else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось найти исходное сообщение.",
        )
            .await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let Some(cache_entry): Option<TranscriptionCache> =
        cache.get::<TranscriptionCache>(&file_cache_key).await?
    else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось найти текст в кеше.",
        )
            .await?;
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
    user_id: u64,
) -> Result<(), MyError> {
    let Some(message) = query.message.and_then(|m| m.regular_message().cloned()) else {
        return Ok(());
    };

    if !is_admin_or_author(
        &bot,
        message.chat.id,
        message.chat.is_group() || message.chat.is_supergroup(),
        &query.from,
        user_id,
    )
        .await
    {
        bot.answer_callback_query(query.id)
            .text("❌ У вас нет прав использовать эту кнопку!")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let Some(audio_message_id) = message.reply_to_message().map(|m| m.id.0) else {
        bot.answer_callback_query(query.id)
            .text("❌ Не удалось найти исходное сообщение для повторной попытки.")
            .show_alert(true)
            .await?;
        return Ok(());
    };

    let cache = config.get_redis_client();
    let message_file_map_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_file_map_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Кнопка устарела.")
            .await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let mut cache_entry = match cache.get::<TranscriptionCache>(&file_cache_key).await? {
        Some(entry) => entry,
        None => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "❌ Не удалось найти исходное аудио.",
            )
                .await?;
            return Ok(());
        }
    };

    if let Some(cached_summary) = cache_entry.summary {
        let final_text = format!(
            "✨:\n<blockquote expandable>{}</blockquote>",
            cached_summary
        );
        bot.edit_message_text(message.chat.id, message.id, final_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(create_summary_keyboard())
            .await?;
        return Ok(());
    }

    if let Some(text) = message.text() && text.contains("[no speech]") {
        bot.edit_message_text(message.chat.id, message.id, "❌ Нельзя составить краткое содержание из аудио без речи.")
            .parse_mode(ParseMode::Html)
            .reply_markup(create_summary_keyboard())
            .await?;
        return Ok(());
    }

    bot.edit_message_text(
        message.chat.id,
        message.id,
        "Составляю краткое содержание...",
    )
        .await?;

    let file_data_result = save_file_to_memory(&bot, &cache_entry.file_id).await;
    let new_summary_result = match file_data_result {
        Ok(file_data) => summarize_audio(cache_entry.mime_type.clone(), file_data, config.clone()).await,
        Err(e) => {
            error!("Failed to download file for summarization: {:?}", e);
            Err(e)
        }
    };

    match new_summary_result {
        Ok(new_summary) if !new_summary.is_empty() && !new_summary.contains("Не удалось получить") => {
            cache_entry.summary = Some(new_summary.clone());
            cache.set(&file_cache_key, &cache_entry, 86400).await?;

            let final_text = format!("✨:\n<blockquote expandable>{}</blockquote>", html::escape(&new_summary));
            bot.edit_message_text(message.chat.id, message.id, final_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(create_summary_keyboard())
                .await?;
        }
        _ => {
            let error_text = "❌ Не удалось составить краткое содержание.";
            let retry_keyboard = create_retry_keyboard(audio_message_id, user_id, "summarize", cache_entry.attempt);

            bot.edit_message_text(
                message.chat.id,
                message.id,
                error_text,
            )
                .reply_markup(retry_keyboard)
                .await?;
        }
    }

    Ok(())
}

async fn get_cached(
    bot: &Bot,
    file: &AudioStruct,
    config: &Config,
    force_no_cache: bool,
) -> Result<TranscriptionCache, MyError> {
    let cache = config.get_redis_client();
    let file_cache_key = format!("transcription_by_file:{}", &file.file_unique_id);

    if !force_no_cache
        && let Some(cached_text) = cache.get::<TranscriptionCache>(&file_cache_key).await?
            && !cached_text.full_text.is_empty() {
                debug!("File cache HIT for unique_id: {}", &file.file_unique_id);
                return Ok(cached_text);
            }

    let file_data = save_file_to_memory(bot, &file.file_id).await?;
    let transcription = Transcription {
        mime_type: file.mime_type.to_string(),
        data: file_data,
        config: config.clone(),
    };
    let processed_parts = transcription.to_text().await;

    if processed_parts.is_empty() || processed_parts[0].contains("Не удалось преобразовать")
    {
        let error_message = processed_parts.first().cloned().unwrap_or_default();
        return Err(MyError::Other(error_message));
    }

    let full_text = processed_parts.join("\n\n");
    let new_cache_entry = TranscriptionCache {
        full_text,
        summary: None,
        file_id: file.file_id.clone(),
        mime_type: file.mime_type.clone(),
        attempt: 0
    };

    cache.set(&file_cache_key, &new_cache_entry, 86400).await?;
    debug!(
        "Saved new transcription to file cache for unique_id: {}",
        file.file_unique_id
    );

    Ok(new_cache_entry)
}

pub async fn transcription_handler(
    bot: Bot,
    msg: &Message,
    config: &Config,
) -> Result<(), MyError> {
    let message = bot
        .send_message(msg.chat.id, "Обрабатываю аудио...")
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(ParseMode::Html)
        .await
        .ok();

    let Some(message) = message else {
        return Ok(());
    };

    let Some(user) = msg.from.as_ref() else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось определить пользователя.",
        )
            .await?;
        return Ok(());
    };

    if let Some(file) = get_file_id(msg).await {
        let cache = config.get_redis_client();
        let message_file_map_key = format!("message_file_map:{}", message.id);

        cache
            .set(&message_file_map_key, &file.file_unique_id, 86400)
            .await?;

        let file_cache_key = format!("transcription_by_file:{}", &file.file_unique_id);
        let empty_cache = TranscriptionCache {
            full_text: String::new(),
            summary: None,
            file_id: file.file_id.clone(),
            mime_type: file.mime_type.clone(),
            attempt: 0
        };
        cache.set(&file_cache_key, &empty_cache, 86400).await?;

        match get_cached(&bot, &file, config, false).await {
            Ok(cache_entry) => {
                let text_parts = split_text(&cache_entry.full_text, 4000);
                if text_parts.is_empty() {
                    bot.edit_message_text(message.chat.id, message.id, "❌ Получен пустой текст.")
                        .await?;
                    return Ok(());
                }

                let keyboard = create_transcription_keyboard(0, text_parts.len(), user.id.0);
                bot.edit_message_text(
                    msg.chat.id,
                    message.id,
                    format!("<blockquote expandable>{}</blockquote>", html::escape(&text_parts[0])),
                )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
            }
            Err(e) => {
                error!("Failed to get transcription: {:?}", e);
                let error_text = match e {
                    MyError::Other(msg) if msg.contains("❌ Не удалось преобразовать") => {
                        msg
                    }
                    MyError::Reqwest(_) => {
                        "❌ Ошибка: Не удалось скачать файл. Возможно, он слишком большой (>20MB)."
                            .to_string()
                    }
                    _ => "❌ Произошла неизвестная ошибка при обработке аудио.".to_string(),
                };

                let original_message_id = msg.id.0;
                let retry_keyboard = create_retry_keyboard(original_message_id, user.id.0, "transcribe", 0); // can have an error

                bot.edit_message_text(message.chat.id, message.id, error_text)
                    .reply_markup(retry_keyboard)
                    .await?;
            }
        }
    } else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось найти голосовое сообщение.",
        )
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

pub async fn retry_speech_handler(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
    _original_message_id: i32,
    user_id: u64,
    action_type: &str,
    attempt: u32
) -> Result<(), MyError> {
    let Some(message) = query.message.and_then(|m| m.regular_message().cloned()) else {
        return Ok(());
    };

    if query.from.id.0 != user_id {
        bot.answer_callback_query(query.id)
            .text("❌ Вы не можете использовать эту кнопку.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    bot.answer_callback_query(query.id).await?;

    if attempt >= 1 {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Лимит попыток исчерпан.",
        )
            .await?;
        return Ok(());
    }

    let bot_message_id = message.id.0;
    let new_attempt = attempt + 1;

    let Some(replied_to_audio_message_id) = message.reply_to_message().map(|m| m.id.0) else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось найти исходное сообщение для повторной попытки.",
        )
            .await?;
        return Ok(());
    };

    let cache = config.get_redis_client();
    let message_file_map_key = format!("message_file_map:{}", bot_message_id);

    let Some(file_unique_id): Option<String> = cache.get::<String>(&message_file_map_key).await? else {
        bot.edit_message_text(message.chat.id, message.id, "❌ Кэш для повторной попытки не найден.")
            .await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);

    match action_type {
        "transcribe" => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "🔁 Повторная обработка аудио...",
            )
                .await?;

            let Some(cache_entry_template): Option<TranscriptionCache> = cache.get(&file_cache_key).await? else {
                info!("{}", &file_cache_key);
                bot.edit_message_text(message.chat.id, message.id, "❌ Не удалось получить метаданные аудио.").await?;
                return Ok(());
            };

            let file = AudioStruct {
                mime_type: cache_entry_template.mime_type,
                file_id: cache_entry_template.file_id,
                file_unique_id: file_unique_id.clone(),
            };

            cache.delete(&file_cache_key).await?;

            match get_cached(&bot, &file, config, true).await {
                Ok(cache_entry) => {
                    let text_parts = split_text(&cache_entry.full_text, 4000);

                    let keyboard = create_transcription_keyboard(0, text_parts.len(), user_id);
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        format!("<blockquote expandable>{}</blockquote>", html::escape(&text_parts[0])),
                    )
                        .parse_mode(ParseMode::Html)
                        .reply_markup(keyboard)
                        .await?;
                }
                Err(e) => {
                    error!("Failed to get transcription on retry: {:?}", e);
                    let error_text = match e {
                        MyError::Other(msg) if msg.contains("❌ Не удалось преобразовать") => {
                            msg
                        }
                        MyError::Reqwest(_) => {
                            "❌ Ошибка: Не удалось скачать файл. Возможно, он слишком большой (>20MB)."
                                .to_string()
                        }
                        _ => "❌ Произошла неизвестная ошибка при обработке аудио.".to_string(),
                    };

                    let retry_keyboard = create_retry_keyboard(replied_to_audio_message_id, user_id, "transcribe", new_attempt);

                    bot.edit_message_text(message.chat.id, message.id, error_text)
                        .reply_markup(retry_keyboard)
                        .await?;

                    let empty_cache = TranscriptionCache {
                        full_text: String::new(),
                        summary: None,
                        file_id: file.file_id.clone(),
                        mime_type: file.mime_type.clone(),
                        attempt: new_attempt
                    };
                    cache.set(&file_cache_key, &empty_cache, 86400).await?;
                }
            }
        }
        "summarize" => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "🔁 Повторное составление краткого содержания...",
            )
                .await?;

            let mut cache_entry = match cache.get::<TranscriptionCache>(&file_cache_key).await? {
                Some(entry) => entry,
                None => {
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        "❌ Не удалось найти исходное аудио.",
                    )
                        .await?;
                    return Ok(());
                }
            };

            cache_entry.summary = None;

            let file_data_result = save_file_to_memory(&bot, &cache_entry.file_id).await;
            let new_summary_result = match file_data_result {
                Ok(file_data) => summarize_audio(cache_entry.mime_type.clone(), file_data, config.clone()).await,
                Err(e) => {
                    error!("Failed to download file for summarization on retry: {:?}", e);
                    Err(e)
                }
            };

            match new_summary_result {
                Ok(new_summary) if !new_summary.is_empty() && !new_summary.contains("Не удалось получить") => {
                    cache_entry.summary = Some(new_summary.clone());
                    cache_entry.attempt = 0;
                    cache.set(&file_cache_key, &cache_entry, 86400).await?;

                    let final_text = format!("✨:\n<blockquote expandable>{}</blockquote>", html::escape(&new_summary));
                    bot.edit_message_text(message.chat.id, message.id, final_text)
                        .parse_mode(ParseMode::Html)
                        .reply_markup(create_summary_keyboard())
                        .await?;
                }
                _ => {
                    let error_text = "❌ Не удалось составить краткое содержание.";

                    cache_entry.attempt = new_attempt;

                    let retry_keyboard = create_retry_keyboard(replied_to_audio_message_id, user_id, "summarize", cache_entry.attempt);

                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        error_text,
                    )
                        .reply_markup(retry_keyboard)
                        .await?;

                    cache_entry.summary = None;
                    cache.set(&file_cache_key, &cache_entry, 86400).await?;
                }
            }
        }
        _ => {
            log::warn!("Unknown action type for retry: {}", action_type);
            bot.edit_message_text(message.chat.id, message.id, "❌ Неизвестный тип действия для повторной попытки.")
                .await?;
        }
    }

    Ok(())
}

pub async fn summarize_audio(
    mime_type: String,
    data: Bytes,
    config: Config,
) -> Result<String, MyError> {
    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

    let ai_model = config.get_json_config().get_ai_model().to_owned();
    let prompt = config.get_json_config().get_summarize_prompt().to_owned();

    let mut context = Context::new();
    context.push_message(Role::Model, prompt);

    let mut client = GemSession::Builder()
        .model(Models::Custom(ai_model))
        .timeout(Some(Duration::from_secs(120)))
        .context(context)
        .build();

    let response = client
        .send_blob(Blob::new(&mime_type, &data), Role::User, &settings)
        .await?;

    Ok(response
        .get_results()
        .first()
        .cloned()
        .unwrap_or_else(|| "❌ Не удалось получить краткое содержание.".to_string()))
}

pub async fn get_file_id(msg: &Message) -> Option<AudioStruct> {
    match &msg.kind {
        MessageKind::Common(common) => match &common.media_kind {
            teloxide::types::MediaKind::Audio(audio) => Some(AudioStruct {
                mime_type: audio.audio.mime_type.as_ref()?.essence_str().to_owned(),
                file_id: audio.audio.file.id.0.to_string(),
                file_unique_id: audio.audio.file.unique_id.0.to_string(),
            }),
            teloxide::types::MediaKind::Voice(voice) => Some(AudioStruct {
                mime_type: voice.voice.mime_type.as_ref()?.essence_str().to_owned(),
                file_id: voice.voice.file.id.0.to_owned(),
                file_unique_id: voice.voice.file.unique_id.0.to_owned(),
            }),
            teloxide::types::MediaKind::VideoNote(video_note) => Some(AudioStruct {
                mime_type: "video/mp4".to_owned(),
                file_id: video_note.video_note.file.id.0.to_owned(),
                file_unique_id: video_note.video_note.file.unique_id.0.to_owned(),
            }),
            _ => None,
        },
        _ => None,
    }
}

pub async fn save_file_to_memory(bot: &Bot, file_id: &str) -> Result<Bytes, MyError> {
    let file = bot.get_file(FileId(file_id.to_string())).send().await?;
    let file_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.token(),
        file.path
    );
    let response = reqwest::get(file_url).await?;
    Ok(response.bytes().await?)
}

impl Transcription {
    pub async fn to_text(&self) -> Vec<String> {
        let mut settings = Settings::new();
        settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

        let error_answer = "❌ Не удалось преобразовать текст из сообщения.".to_string();
        let ai_model = self.config.get_json_config().get_ai_model().to_owned();
        let prompt = self.config.get_json_config().get_ai_prompt().to_owned();

        let mut context = Context::new();
        context.push_message(Role::Model, prompt);

        let mut client = GemSession::Builder()
            .model(Models::Custom(ai_model))
            .timeout(Some(Duration::from_secs(120)))
            .context(context)
            .build();

        let mut attempts = 0;
        let mut last_error = String::new();

        while attempts < 3 {
            match client
                .send_blob(
                    Blob::new(&self.mime_type, &self.data),
                    Role::User,
                    &settings,
                )
                .await
            {
                Ok(response) => {
                    let full_text = response.get_results().first().cloned().unwrap_or_default();
                    if !full_text.is_empty() {
                        return split_text(&full_text, 4000);
                    }
                    attempts += 1;
                    info!("Received empty response, attempt {}", attempts);
                }
                Err(error) => {
                    attempts += 1;
                    let error_string = error.to_string();
                    if error_string == last_error {
                        continue;
                    }
                    last_error = error_string;
                    error!("Transcription error (attempt {}): {:?}", attempts, error);
                }
            }
        }
        vec![error_answer + "\n\n" + &last_error]
    }
}