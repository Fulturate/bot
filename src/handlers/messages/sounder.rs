use crate::config::Config;
use crate::handlers::messages::sound::audio::audio_handler;
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

pub(crate) async fn sounds_handlers(
    bot: Bot,
    message: Message,
    config: &Config,
) -> Result<(), MyError> {
    let sound_enum = if message.voice().is_some() {
        SoundEnum::Voice
    } else if message.video_note().is_some() {
        SoundEnum::VideoNote
    } else if message.audio().is_some() {
        SoundEnum::Audio
    } else {
        return Ok(());
    };

    match sound_enum {
        SoundEnum::Audio => audio_handler(bot, message, config).await,
        SoundEnum::Voice => voice_handler(bot, message, config).await,
        SoundEnum::VideoNote => voice_note_handler(bot, message, config).await,
    }
}
