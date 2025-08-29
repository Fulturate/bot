use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "start command? :D")]
    Start(String),
    #[command(description = "Speech recognition", alias = "sr")]
    SpeechRecognition,
    #[command(description = "Translate", alias = "tr")]
    Translate(String),
    #[command(parse_with = "split", description = "Set currency to convert")]
    SetCurrency { code: String },
    #[command(description = "List of available currencies to convert")]
    ListCurrency,
    #[command(description = "Settings of bot")]
    Settings,
}

pub struct AudioStruct {
    pub mime_type: String,
    pub file_id: String,
    pub file_unique_id: String,
}
