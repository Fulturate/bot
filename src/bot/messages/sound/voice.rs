use crate::config::Config;
use crate::errors::MyError;
use crate::util::transcription::transcription_handler;
use teloxide::prelude::*;

pub async fn voice_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), MyError> {
    transcription_handler(bot, msg, config).await
}
