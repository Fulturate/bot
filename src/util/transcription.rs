use crate::config::Config;
use crate::util::errors::MyError;
use base64::encode;
use bytes::Bytes;
use gemini_client_rs::types::{Candidate, Content, ContentPart, GenerateContentRequest, PartResponse, Role};
use std::error::Error;
use std::io::Read;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Request as TeloxideRequest, Requester};
use teloxide::types::{Message, MessageKind, ParseMode};
use teloxide::Bot;

pub async fn transcription_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    if let Some(file_id) = get_file_id(&msg) {
        let file_data = save_file_to_memory(&bot, &file_id).await?;
        let transcription = Transcription {
            data: file_data,
            config: config.clone(),
        };
        let text = transcription.to_text().await;

        bot.send_message(
            msg.chat.id,
            format!(
                "Транскрипция: {}",
                text.unwrap_or_else(|e| format!("Ошибка: {}", e))
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    } else {
        bot.send_message(msg.chat.id, "Не удалось найти голосовое сообщение.")
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

pub fn get_file_id(msg: &Message) -> Option<String> {
    match &msg.kind {
        MessageKind::Common(common) => match &common.media_kind {
            teloxide::types::MediaKind::Audio(audio) => Some(audio.audio.file.id.clone()),
            teloxide::types::MediaKind::Voice(voice) => Some(voice.voice.file.id.clone()),
            teloxide::types::MediaKind::VideoNote(video_note) => {
                Some(video_note.video_note.file.id.clone())
            }
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
    pub(crate) data: Bytes,
    pub(crate) config: Config,
}

impl Transcription {
    pub async fn to_text(self) -> Result<String, Box<dyn Error>> {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![ContentPart::Text(
                    "Сделай транскрипцию голосового сообщения:\n".to_owned() + self.data.,
                )],
                role: Role::User,
            }],
            tools: None,
        };

        let response = self
            .config
            .get_client()
            .generate_content("gemini-2.0-flash-exp", &request)
            .await?;

        let response_string = Self::candidate_to_string(response.candidates);

        println!("{}", response_string);
        Ok(response_string)
    }

    fn candidate_to_string(candidates: Option<Vec<Candidate>>) -> String {
        let mut response = String::new();
        if let Some(candidates) = candidates {
            for candidate in candidates {
                for part in candidate.content.parts {
                    if let PartResponse::Text(text) = part {
                        response = text.clone();
                    }
                }
            }
        }
        response
    }
}
