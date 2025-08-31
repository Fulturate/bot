use super::enums::AudioStruct;
use crate::config::Config;
use crate::util::errors::MyError;
use bytes::Bytes;
use gem_rs::types::HarmBlockThreshold;
use log::{debug, error, info};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use teloxide::Bot;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters, SendMessageSetters};
use teloxide::requests::{Request as TeloxideRequest, Requester};
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageKind, ParseMode,
    ReplyParameters,
};

#[derive(Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs, Clone)]
struct TranscriptionCache {
    full_text: String,
    summary: Option<String>,
    file_id: String,
    mime_type: String,
}

async fn get_cached(
    bot: &Bot,
    file: &AudioStruct,
    config: &Config,
) -> Result<TranscriptionCache, MyError> {
    let cache = config.get_redis_client();
    let file_cache_key = format!("transcription_by_file:{}", &file.file_unique_id);

    if let Some(cached_text) = cache.get::<TranscriptionCache>(&file_cache_key).await? {
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
        let error_message = processed_parts.get(0).cloned().unwrap_or_default();
        return Err(MyError::Other(error_message));
    }

    let full_text = processed_parts.join("\n\n");

    let new_cache_entry = TranscriptionCache {
        full_text,
        summary: None,
        file_id: file.file_id.clone(),
        mime_type: file.mime_type.clone(),
    };

    cache.set(&file_cache_key, &new_cache_entry, 86400).await?;
    debug!(
        "Saved new transcription to file cache for unique_id: {}",
        file.file_id
    );

    Ok(new_cache_entry)
}

pub async fn transcription_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    let message = bot
        .send_message(msg.chat.id, "Обрабатываю аудио...")
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(ParseMode::Html)
        .await
        .ok();

    let Some(message) = message else {
        return Ok(());
    };

    if let Some(file) = get_file_id(&msg).await {
        match get_cached(&bot, &file, config).await {
            Ok(cache_entry) => {
                let cache = config.get_redis_client();

                let message_file_map_key = format!("message_file_map:{}", message.id);
                cache
                    .set(&message_file_map_key, &file.file_unique_id, 3600)
                    .await?;

                let text_parts = split_text(cache_entry.full_text, 4000);

                let keyboard = InlineKeyboardMarkup::new(vec![vec![
                    InlineKeyboardButton::callback("✨", "summarize"),
                    InlineKeyboardButton::callback(
                        "🗑️",
                        format!("delete_{}", msg.from.unwrap().id.0),
                    ),
                ]]);

                bot.edit_message_text(
                    msg.chat.id,
                    message.id,
                    format!("<blockquote expandable>{}</blockquote>", text_parts[0]),
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard.clone())
                .await?;

                for part in text_parts.iter().skip(1) {
                    bot.send_message(
                        msg.chat.id,
                        format!("<blockquote expandable>\n{}\n</blockquote>", part),
                    )
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard.clone())
                    .await?;
                }
            }
            Err(e) => {
                error!("Failed to get transcription: {:?}", e);
                let error_text = match e {
                    MyError::Other(msg) if msg.contains("Не удалось преобразовать") => {
                        msg
                    }
                    MyError::Reqwest(_) => {
                        "❌ Ошибка: Не удалось скачать файл. Возможно, он слишком большой (>20MB)."
                            .to_string()
                    }
                    _ => "❌ Произошла неизвестная ошибка при обработке аудио.".to_string(),
                };
                bot.edit_message_text(message.chat.id, message.id, error_text)
                    .await?;
            }
        }
    } else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "Не удалось найти голосовое сообщение.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }
    Ok(())
}

pub async fn back_handler(bot: Bot, query: CallbackQuery, config: &Config) -> Result<(), MyError> {
    let Some(message) = query.message else {
        return Ok(());
    };
    bot.answer_callback_query(query.id).await?;

    let cache = config.get_redis_client();
    let message_cache_key = format!("message_file_map:{}", message.id());

    let Some(file_unique_id) = cache.get::<String>(&message_cache_key).await? else {
        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "❌ Не удалось найти исходное сообщение.",
        )
        .await?;
        return Ok(());
    };

    let file_cache_key = format!("transcription_by_file:{}", file_unique_id);
    let Some(cache_entry) = cache.get::<TranscriptionCache>(&file_cache_key).await? else {
        bot.edit_message_text(
            message.chat().id,
            message.id(),
            "❌ Не удалось найти исходный текст в кеше.",
        )
        .await?;
        return Ok(());
    };

    let text_parts = split_text(cache_entry.full_text, 4000);

    bot.edit_message_text(
        message.chat().id,
        message.id(),
        format!("<blockquote expandable>{}</blockquote>", text_parts[0]),
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✨", "summarize"),
        InlineKeyboardButton::callback("🗑️", format!("delete_{}", query.from.id.0)),
    ]]))
    .await?;

    Ok(())
}

pub async fn summarization_handler(
    bot: Bot,
    query: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = query.message else {
        return Ok(());
    };

    let Some(message) = message.regular_message() else {
        return Ok(());
    };

    bot.answer_callback_query(query.id).await?;

    let cache = config.get_redis_client();

    let message_file_map_key = format!("message_file_map:{}", message.id);
    let Some(file_unique_id) = cache.get::<String>(&message_file_map_key).await? else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось обработать запрос. Кнопка устарела.",
        )
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
            "Краткое содержание:\n<blockquote expandable>{}</blockquote>",
            cached_summary
        );

        let back_keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            "back_to_full",
        )]]);

        bot.edit_message_text(message.chat.id, message.id, final_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(back_keyboard)
            .await?;

        return Ok(());
    }

    bot.edit_message_text(
        message.chat.id,
        message.id,
        "Составляю краткое содержание...",
    )
    .await?;

    let file_data = save_file_to_memory(&bot, &cache_entry.file_id).await?;
    let new_summary =
        summarize_audio(cache_entry.mime_type.clone(), file_data, config.clone()).await?;

    if new_summary.is_empty() || new_summary.contains("Не удалось получить") {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "❌ Не удалось составить краткое содержание.",
        )
        .await?;
        return Ok(());
    }

    cache_entry.summary = Some(new_summary.clone());
    cache.set(&file_cache_key, &cache_entry, 86400).await?;

    let final_text = format!(
        "Краткое содержание:\n<blockquote expandable>{}</blockquote>",
        new_summary
    );

    let back_keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        "back_to_full",
    )]]);

    bot.edit_message_text(message.chat.id, message.id, final_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(back_keyboard)
        .await?;

    Ok(())
}

async fn summarize_audio(
    mime_type: String,
    data: Bytes,
    config: Config,
) -> Result<String, MyError> {
    let mut settings = gem_rs::types::Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

    let ai_model = config.get_json_config().get_ai_model().to_owned();
    let prompt = config.get_json_config().get_summarize_prompt().to_owned();

    let mut context = gem_rs::types::Context::new();
    context.push_message(gem_rs::types::Role::Model, prompt);

    let mut client = gem_rs::client::GemSession::Builder()
        .model(gem_rs::api::Models::Custom(ai_model))
        .timeout(Some(Duration::from_secs(120)))
        .context(context)
        .build();

    let response = client
        .send_blob(
            gem_rs::types::Blob::new(&mime_type, &data),
            gem_rs::types::Role::User,
            &settings,
        )
        .await?;

    Ok(response
        .get_results()
        .first()
        .cloned()
        .unwrap_or_else(|| "Не удалось получить краткое содержание.".to_string()))
}

pub async fn get_file_id(msg: &Message) -> Option<AudioStruct> {
    match &msg.kind {
        MessageKind::Common(common) => match &common.media_kind {
            teloxide::types::MediaKind::Audio(audio) => Some(AudioStruct {
                mime_type: audio
                    .audio
                    .mime_type
                    .as_ref()
                    .unwrap()
                    .essence_str()
                    .to_owned(),
                file_id: audio.audio.file.id.0.to_string(),
                file_unique_id: audio.audio.file.unique_id.0.to_string(),
            }),
            teloxide::types::MediaKind::Voice(voice) => Some(AudioStruct {
                mime_type: voice
                    .voice
                    .mime_type
                    .as_ref()
                    .unwrap()
                    .essence_str()
                    .to_owned(),
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
    let file = bot
        .get_file(teloxide::types::FileId(file_id.to_string()))
        .send()
        .await?;
    let file_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.token(),
        file.path
    );

    let response = reqwest::get(file_url).await?;
    let bytes = response.bytes().await?;

    Ok(bytes)
}

pub struct Transcription {
    pub(crate) mime_type: String,
    pub(crate) data: Bytes,
    pub(crate) config: Config,
}

impl Transcription {
    pub async fn to_text(&self) -> Vec<String> {
        let mut settings = gem_rs::types::Settings::new();
        settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

        let error_answer = "❌ Не удалось преобразовать текст из сообщения.".to_string();

        let ai_model = self.config.get_json_config().get_ai_model().to_owned();
        let prompt = self.config.get_json_config().get_ai_prompt().to_owned();

        let mut context = gem_rs::types::Context::new();
        context.push_message(gem_rs::types::Role::Model, prompt);

        let mut client = gem_rs::client::GemSession::Builder()
            .model(gem_rs::api::Models::Custom(ai_model))
            .timeout(Some(Duration::from_secs(120)))
            .context(context)
            .build();

        let mut attempts = 0;
        let mut last_text = String::new();
        let mut last_error = String::new();

        while attempts < 3 {
            match client
                .send_blob(
                    gem_rs::types::Blob::new(&self.mime_type, &self.data),
                    gem_rs::types::Role::User,
                    &settings,
                )
                .await
            {
                Ok(response) => {
                    let full_text = response
                        .get_results()
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "".to_string());

                    if full_text == last_text && !full_text.is_empty() {
                        continue;
                    }
                    last_text = full_text.clone();

                    if !full_text.is_empty() {
                        return split_text(full_text, 4000);
                    } else {
                        attempts += 1;
                        info!(
                            "Received empty successful response from transcription service, attempt {}",
                            attempts
                        );
                    }
                }
                Err(error) => {
                    attempts += 1;

                    if error.to_string() == last_error {
                        continue;
                    }
                    last_error = error.to_string();

                    error!(
                        "Error with transcription (attempt {}): {:?}",
                        attempts, error
                    );
                }
            }
        }
        vec![error_answer + "\n\n" + &last_error]
    }
}

fn split_text(text: String, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}
