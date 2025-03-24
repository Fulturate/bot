use crate::config::Config;
use crate::loader::Error;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn audio_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), Error> {
    bot.send_message(msg.chat.id, "Sound?")
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
