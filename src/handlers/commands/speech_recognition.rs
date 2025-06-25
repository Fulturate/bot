use crate::config::Config;
use crate::util::{errors::MyError, transcription::transcription_handler};
use teloxide::prelude::*;

pub async fn speech_recognition_handler(
    bot: Bot,
    msg: Message,
    config: &Config,
) -> Result<(), MyError> {
    if msg.reply_to_message().is_some() {
        transcription_handler(bot, msg.reply_to_message().unwrap().clone(), config).await?;
    }
    // msg.reply_to_message().is_some().then(async || {
    //     transcription_handler(bot, msg.reply_to_message().unwrap().clone(), config).await
    // });
    Ok(())
}
