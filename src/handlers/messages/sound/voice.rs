use crate::config::Config;
use crate::loader::Error;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn voice_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), Error> {
    bot.send_message(msg.chat.id, "Voice?")
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
