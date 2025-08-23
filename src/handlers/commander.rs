use crate::config::Config;
use crate::handlers::commands::{
    settings::{currency_codes_handler, currency_codes_list_handler},
    speech_recognition::speech_recognition_handler,
    start::start_handler,
};
use crate::util::{enums::Command, errors::MyError};
use teloxide::Bot;
use teloxide::prelude::Message;
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
            Command::SetCurrency { code } => currency_codes_handler(bot, message, code).await,
            Command::ListCurrency => currency_codes_list_handler(bot, message).await,
        }
    });
    Ok(())
}
