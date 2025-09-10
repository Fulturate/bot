use crate::core::config::Config;
use crate::errors::MyError;
use crate::util::transcription::transcription_handler;
use teloxide::prelude::*;

pub async fn speech_recognition_handler(
    bot: Bot,
    msg: Message,
    config: &Config,
) -> Result<(), MyError> {
    if msg.reply_to_message().is_some() {
        transcription_handler(bot, msg.reply_to_message().unwrap().clone(), config).await?;
    }
    Ok(())
}
