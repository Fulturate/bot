use crate::{
    bot::messages::sound::{voice::voice_handler, voice_note::voice_note_handler},
    core::config::Config,
    errors::MyError,
};
use teloxide::{Bot, prelude::Message};

pub async fn sound_handlers(bot: Bot, message: Message, config: &Config) -> Result<(), MyError> {
    let config = config.clone();
    tokio::spawn(async move {
        if message.voice().is_some() {
            voice_handler(bot, message, &config).await
        } else if message.video_note().is_some() {
            voice_note_handler(bot, message, &config).await
        } else {
            Ok(())
        }
    });
    Ok(())
}
