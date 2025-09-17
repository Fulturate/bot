use crate::core::config::Config;
use crate::core::services::speech_recognition::transcription_handler;
use crate::errors::MyError;
use teloxide::prelude::*;

pub async fn voice_note_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    transcription_handler(bot, &msg, config).await
}
