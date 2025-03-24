use crate::config::Config;
use crate::handlers::messages::sounder::sounds_handlers;
use crate::loader::Error;
use teloxide::prelude::Message;
use teloxide::Bot;

pub enum Audio {
    Voice,
    Sound,
    Audio,
}

pub(crate) async fn messages_handlers(bot: Bot, message: Message) -> Result<(), Error> {
    let config = Config::new().await;
    if message.voice().is_some() || message.video_note().is_some() || message.audio().is_some() {
        sounds_handlers(bot, message, &config).await
    } else {
        Ok(())
    }
}
