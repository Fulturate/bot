use crate::config::Config;
use crate::util::errors::MyError;
use bytes::Bytes;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Request as TeloxideRequest, Requester};
use teloxide::types::{Message, MessageKind, ParseMode};

use super::enums::AudioStruct;

pub async fn transcription_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    if let Some(file) = get_file_id(&msg) {
        let file_data = save_file_to_memory(&bot, &file.file_id).await?;
        let transcription = Transcription {
            mime_type: file.mime_type,
            data: file_data,
            config: config.clone(),
        };

        let message = bot
            .send_message(msg.chat.id, "Подождите...")
            .parse_mode(ParseMode::Html)
            .await
            .ok();

        let text = transcription.to_text().await;

        bot.edit_message_text(
            msg.chat.id,
            message.unwrap().id,
            format!(
                "Транскрипция: {}",
                text.first().unwrap_or(&String::from("Нет данных"))
            ),
        )
        .await?;
    } else {
        bot.send_message(msg.chat.id, "Не удалось найти голосовое сообщение.")
            .parse_mode(ParseMode::Html)
            .await?;
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
        // self.data
        // let content: Vec<u8> = fs::read("audio.ogg").unwrap();
        let settings = gem_rs::types::Settings::new();

        let mut client = gem_rs::client::GemSession::Builder()
            .model(gem_rs::api::Models::Custom(
                "gemini-2.0-flash-thinking-exp".to_owned(),
            ))
            .build();
        // let ap = gem_rs::client::GemSession::from(self.config.get_client())
        let response = client
            .send_message_with_blob(
                "Ретранслируй голосовое сообщение, при этом соблюдая оригинальный язык голосовухи",
                gem_rs::types::Blob::new(&self.mime_type, &self.data),
                gem_rs::types::Role::User,
                &settings,
            )
            .await
            .unwrap();

        response.get_results()
        // results[0];

        // println!("{:#?}", results)
    }
    // pub async fn to_text(self) -> Result<String, Box<dyn Error>> {
    //     let request = GenerateContentRequest {
    //         contents: vec![Content {
    //             parts: vec![ContentPart::Text(
    //                 "Сделай транскрипцию голосового сообщения:\n".to_owned() + self.data.,
    //             )],
    //             role: Role::User,
    //         }],
    //         tools: None,
    //     };

    //     let response = self
    //         .config
    //         .get_client()
    //         .generate_content("gemini-2.0-flash-exp", &request)
    //         .await?;

    //     let response_string = Self::candidate_to_string(response.candidates);

    //     println!("{}", response_string);
    //     Ok(response_string)
    // }

    // fn candidate_to_string(candidates: Option<Vec<Candidate>>) -> String {
    //     let mut response = String::new();
    //     if let Some(candidates) = candidates {
    //         for candidate in candidates {
    //             for part in candidate.content.parts {
    //                 if let PartResponse::Text(text) = part {
    //                     response = text.clone();
    //                 }
    //             }
    //         }
    //     }
    //     response
    // }
}
