use crate::config::Config;
use crate::util::errors::MyError;
use teloxide::prelude::*;

pub async fn speech_recognition_handler(
    bot: Bot,
    message: Message,
    _: &Config,
) -> Result<(), MyError> {
    bot.send_message(message.chat.id, "Soon..").await?;
    Ok(())
}
