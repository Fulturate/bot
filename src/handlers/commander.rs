use crate::config::Config;
use crate::handlers::commands::{
    speech_recognition::speech_recognition_handler, start::start_handler,
};
use crate::util::{enums::Command, errors::MyError};
use teloxide::prelude::Message;
use teloxide::Bot;
use tokio::task;

pub(crate) async fn command_handlers(
    bot: Bot,
    message: Message,
    cmd: Command,
) -> Result<(), MyError> {
    let config = Config::new().await;
    task::spawn(async move {
        match cmd {
            Command::Start => start_handler(bot, message, &config).await,
            Command::SpeechRecognition => speech_recognition_handler(bot, message, &config).await,
        }
    });
    Ok(())
}
