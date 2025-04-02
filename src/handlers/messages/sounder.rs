use crate::config::Config;
use crate::handlers::messages::sound::voice::voice_handler;
use crate::handlers::messages::sound::voice_note::voice_note_handler;
use crate::util::errors::MyError;
use teloxide::prelude::Message;
use teloxide::Bot;

pub enum SoundEnum {
    Voice,
    VideoNote,
    Audio,
}

pub(crate) async fn sound_handlers(
    bot: Bot,
    message: Message,
    config: &Config,
) -> Result<(), MyError> {
    let config = config.clone();
    tokio::spawn(async move {
        let sound = match (message.voice(), message.video_note(), message.audio()) {
            (Some(_), _, _) => SoundEnum::Voice,
            (_, Some(_), _) => SoundEnum::VideoNote,
            (_, _, Some(_)) => SoundEnum::Audio,
            _ => return Ok(()),
        };

        match sound {
            SoundEnum::Audio => Ok(()),
            SoundEnum::Voice => voice_handler(bot, message, &config).await,
            SoundEnum::VideoNote => voice_note_handler(bot, message, &config).await,
        }
    });
    Ok(())
}