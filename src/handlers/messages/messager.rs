use crate::config::Config;
use crate::handlers::messages::sounder::sound_handlers;
use crate::util::errors::MyError;
use teloxide::prelude::Message;
use teloxide::Bot;
use tokio::task;

pub(crate) async fn messages_handlers(bot: Bot, message: Message) -> Result<(), MyError> {
    let config = Config::new().await;

    task::spawn(async move {
        if message.voice().is_some() || message.video_note().is_some() || message.audio().is_some() {
            sound_handlers(bot, message, &config).await
        } else {
            Ok(())
        }
    });
    Ok(())
}
