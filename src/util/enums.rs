use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "start command? :D")]
    Start,
    #[command(description = "Speech recognition", alias = "sr")]
    SpeechRecognition,
}

pub struct AudioStruct {
    pub mime_type: String,
    pub file_id: String,
}
