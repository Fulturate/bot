use crate::config::Config;
use crate::loader::Error;
use teloxide::prelude::*;

pub async fn speech_recognition_handler(
    bot: Bot,
    message: Message,
    config: &Config,
) -> Result<(), Error> {
    bot.send_message(message.chat.id, "Soon..").await?;
    Ok(())
}
