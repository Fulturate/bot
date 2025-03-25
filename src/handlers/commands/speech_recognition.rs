use crate::config::Config;
use teloxide::prelude::*;
use crate::util::errors::MyError;

pub async fn speech_recognition_handler(
    bot: Bot,
    message: Message,
    _: &Config,
) -> Result<(), MyError> {
    bot.send_message(message.chat.id, "Soon..").await?;
    Ok(())
}
