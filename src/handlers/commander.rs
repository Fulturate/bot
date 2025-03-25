use crate::config::Config;
use crate::handlers::commands::speech_recognition::speech_recognition_handler;
use crate::handlers::commands::start::start_handler;
use crate::util::enums::Command;
use crate::util::errors::MyError;
use teloxide::prelude::Message;
use teloxide::Bot;

pub(crate) async fn command_handlers(
    bot: Bot,
    message: Message,
    cmd: Command,
) -> Result<(), MyError> {
    let config = Config::new().await;
    match cmd {
        Command::Start => start_handler(bot, message, &config).await,
        Command::SpeechRecognition => speech_recognition_handler(bot, message, &config).await,
    }
}
