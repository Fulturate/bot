use crate::config::Config;
use crate::handlers::commands::speech_recognition::speech_recognition_handler;
use crate::handlers::commands::start::start_handler;
use crate::loader::Error;
use teloxide::macros::BotCommands;
use teloxide::prelude::Message;
use teloxide::Bot;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "start command? :D")]
    Start,
    #[command(description = "Speech recognition")]
    SpeechRecognition,
}

pub(crate) async fn command_handlers(
    bot: Bot,
    message: Message,
    cmd: Command,
) -> Result<(), Error> {
    let config = Config::new().await;
    match cmd {
        Command::Start => start_handler(bot, message, &config).await,
        Command::SpeechRecognition => speech_recognition_handler(bot, message, &config).await,
    }
}
