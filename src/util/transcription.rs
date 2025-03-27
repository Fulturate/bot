use crate::config::Config;
use crate::util::errors::MyError;
use bytes::Bytes;
use std::time::Duration;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::requests::{Request as TeloxideRequest, Requester};
use teloxide::types::{Message, MessageKind, ParseMode, ReplyParameters};
use teloxide::Bot;
use super::enums::AudioStruct;

pub async fn transcription_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    let message = bot.send_message(msg.chat.id, "Обрабатываю аудио...")
        .reply_parameters(ReplyParameters::new(msg.id))
        .parse_mode(ParseMode::Html)
        .await
        .ok();

    if let Some(message) = message {
        if let Some(file) = get_file_id(&msg) {
            let file_data = save_file_to_memory(&bot, &file.file_id).await?;
            let transcription = Transcription {
                mime_type: file.mime_type,
                data: file_data,
                config: config.clone(),
            };

            let text_parts = transcription.to_text().await;

            bot.edit_message_text(
                msg.chat.id,
                message.id,
                format!("<blockquote expandable>\n{}\n</blockquote>", text_parts[0]),
            )
                .parse_mode(ParseMode::Html)
                .await?;

            for part in text_parts.iter().skip(1) {
                bot.send_message(
                    msg.chat.id,
                    format!("<blockquote expandable>\n{}\n</blockquote>", part),
                )
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        } else {
            bot.edit_message_text(
                msg.chat.id,
                message.id,
                "Не удалось найти голосовое сообщение.",
            )
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}

pub fn get_file_id(msg: &Message) -> Option<AudioStruct> {
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
                file_id: audio.audio.file.id.to_owned(),
            }),
            teloxide::types::MediaKind::Voice(voice) => Some(AudioStruct {
                mime_type: voice
                    .voice
                    .mime_type
                    .as_ref()
                    .unwrap()
                    .essence_str()
                    .to_owned(),
                file_id: voice.voice.file.id.to_owned(),
            }),
            teloxide::types::MediaKind::VideoNote(video_note) => Some(AudioStruct {
                mime_type: "video/mp4".to_owned(),
                file_id: video_note.video_note.file.id.to_owned(),
            }),
            _ => None,
        },
        _ => None,
    }
}

pub async fn save_file_to_memory(bot: &Bot, file_id: &str) -> Result<Bytes, MyError> {
    let file = bot.get_file(file_id).send().await?;
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
    pub async fn to_text(self) -> Vec<String> {
        let settings = gem_rs::types::Settings::new();
        let error_answer = "❌ Не удалось преобразовать текст из аудиосообщения.".to_string();

        let ai_model = self.config.get_json_config().get_ai_model().to_owned();
        let prompt = self.config.get_json_config().get_ai_prompt().to_owned();

        let mut client = gem_rs::client::GemSession::Builder()
            .model(gem_rs::api::Models::Custom(ai_model))
            .timeout(Duration::from_secs(120))
            .build();

        let mut attempts = 0;
        let mut last_text = String::new();

        while attempts < 3 {
            match client
                .send_message_with_blob(
                    &prompt,
                    gem_rs::types::Blob::new(&self.mime_type, &self.data),
                    gem_rs::types::Role::User,
                    &settings,
                )
                .await
            {
                Ok(response) => {
                    let full_text = response.get_results().first().unwrap_or(&error_answer).clone();
                    if full_text == last_text {
                        continue;
                    }
                    last_text = full_text.clone();
                    return split_text(full_text, 4000);
                }
                Err(_) => {
                    attempts += 1;
                }
            }
        }
        vec![error_answer]
    }
}

fn split_text(text: String, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}
