use std::error::Error;
use teloxide::Bot;
use teloxide::requests::{Request, Requester};
use teloxide::types::{Message, MessageKind};
use crate::util::errors::MyError;

pub fn get_file_id(msg: &Message) -> Option<String> {
    match &msg.kind {
        MessageKind::Common(common) => match &common.media_kind {
            teloxide::types::MediaKind::Audio(audio) => Some(audio.audio.file.id.clone()),
            teloxide::types::MediaKind::Voice(voice) => Some(voice.voice.file.id.clone()),
            teloxide::types::MediaKind::VideoNote(video_note) => Some(video_note.video_note.file.id.clone()),
            _ => None,
        },
        _ => None,
    }
}

pub async fn save_file_to_memory(bot: &Bot, file_id: &str) -> Result<Vec<u8>, MyError> {
    let file = bot.get_file(file_id).send().await?;
    let file_url = format!("https://api.telegram.org/file/bot{}/{}", bot.token(), file.path);

    let response = reqwest::get(file_url).await?;
    let bytes = response.bytes().await?;

    Ok(bytes.to_vec())
}

pub struct Transcription {
    pub(crate) data: Vec<u8>
}

impl Transcription {
    pub async fn to_text(self) -> Result<String, Box<dyn Error>> {
        let gem_res = gemini_rs::chat("gemini-2.0-flash")
            .send_message(format!("Транскрибируй этот голосовой файл: {:?}", self.data).as_str())
            .await?;

        println!("{}", gem_res.to_string());
        Ok(gem_res.to_string())
    }

}