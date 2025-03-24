use crate::config::Config;
use crate::loader::Error;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn voice_note_handler(bot: Bot, msg: Message, config: &Config) -> Result<(), Error> {
    bot.send_message(msg.chat.id, "Voice note?")
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
