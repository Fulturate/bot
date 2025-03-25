use crate::config::Config;
use crate::handlers::messages::sounder::sounds_handlers;
use teloxide::prelude::Message;
use teloxide::Bot;
use crate::util::errors::MyError;

pub(crate) async fn messages_handlers(bot: Bot, message: Message) -> Result<(), MyError> {
    let config = Config::new().await;
    if message.voice().is_some() || message.video_note().is_some() || message.audio().is_some() {
        sounds_handlers(bot, message, &config).await
    } else {
        Ok(())
    }
}
