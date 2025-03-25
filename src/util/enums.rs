use teloxide::macros::BotCommands;

pub enum Audio {
    Voice,
    Sound,
    Audio,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "start command? :D")]
    Start,
    #[command(description = "Speech recognition")]
    SpeechRecognition,
}

pub struct AudioStruct {
    pub mime_type: String,
    pub file_id: String,
}
