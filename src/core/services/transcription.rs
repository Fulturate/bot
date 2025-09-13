use crate::bot::keyboards::transcription::create_transcription_keyboard;
use crate::core::config::Config;
use crate::errors::MyError;
use crate::util::enums::AudioStruct;
use bytes::Bytes;
use gem_rs::api::Models;
use gem_rs::client::GemSession;
use gem_rs::types::{Blob, Context, HarmBlockThreshold, Role, Settings};
use log::{debug, error, info};
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::{FileId, MessageKind, ParseMode, ReplyParameters};

#[derive(Debug, Serialize, Deserialize, FromRedisValue, ToRedisArgs, Clone)]
pub struct TranscriptionCache {
    pub full_text: String,
    pub summary: Option<String>,
    pub file_id: String,
    pub mime_type: String,
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

    if processed_parts.is_empty() || processed_parts[0].contains("Не удалось преобразовать") {
        let error_message = processed_parts.first().cloned().unwrap_or_default();
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

    let Some(message) = message else { return Ok(()) };
    let Some(user) = msg.from.as_ref() else {
        bot.edit_message_text(message.chat.id, message.id, "Не удалось определить пользователя.").await?;
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
                    format!("<blockquote expandable>{}</blockquote>", text_parts[0]),
                )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
            }
            Err(e) => {
                error!("Failed to get transcription: {:?}", e);
                let error_text = match e {
                    MyError::Other(msg) if msg.contains("Не удалось преобразовать") => msg,
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
        .unwrap_or_else(|| "Не удалось получить краткое содержание.".to_string()))
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
    let file = bot
        .get_file(FileId(file_id.to_string()))
        .send()
        .await?;
    let file_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot.token(),
        file.path
    );
    let response = reqwest::get(file_url).await?;
    Ok(response.bytes().await?)
}

pub struct Transcription {
    pub(crate) mime_type: String,
    pub(crate) data: Bytes,
    pub(crate) config: Config,
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
                .send_blob(Blob::new(&self.mime_type, &self.data), Role::User, &settings)
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

pub fn split_text(text: &str, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect()
}