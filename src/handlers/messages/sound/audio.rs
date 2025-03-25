use crate::config::Config;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use crate::util::errors::MyError;

pub async fn audio_handler(bot: Bot, msg: Message, _: &Config) -> Result<(), MyError> {
    bot.send_message(msg.chat.id, "Audio?")
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
