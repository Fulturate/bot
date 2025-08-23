// use teloxide::macros::BotCommands;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "start command? :D")]
    Start,
    #[command(description = "Speech recognition", alias = "sr")]
    SpeechRecognition,
    #[command(parse_with = "split", description = "Set currency to convert")]
    SetCurrency { code: String },
    #[command(description = "List of available currencies to convert")]
    ListCurrency,
}

pub struct AudioStruct {
    pub mime_type: String,
    pub file_id: String,
}
