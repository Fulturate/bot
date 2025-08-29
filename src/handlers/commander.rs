use crate::config::Config;
use crate::handlers::commands::{
    settings::{currency_codes_handler, currency_codes_list_handler, settings_command_handler},
    speech_recognition::speech_recognition_handler,
    start::start_handler,
};
use crate::util::{enums::Command, errors::MyError};
use teloxide::Bot;
use teloxide::prelude::Message;
use tokio::task;
use crate::handlers::commands::translate::translate_handler;

pub(crate) async fn command_handlers(
    bot: Bot,
    message: Message,
    cmd: Command,
) -> Result<(), MyError> {
    let config = Config::new().await;
    task::spawn(async move {
        match cmd {
            Command::Start(arg) => start_handler(bot, message, &config, arg).await,
            Command::Translate(arg) => translate_handler(bot, message, &config, arg).await,
            Command::SpeechRecognition => speech_recognition_handler(bot, message, &config).await,
            Command::SetCurrency { code } => currency_codes_handler(bot, message, code).await,
            Command::ListCurrency => currency_codes_list_handler(bot, message).await,
            Command::Settings => settings_command_handler(bot, message, &config).await,
        }
    });
    Ok(())
}