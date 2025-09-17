use crate::{
    core::{config::Config, services::speech_recognition::transcription_handler},
    errors::MyError,
};
use teloxide::{prelude::*, types::ReplyParameters};

pub async fn speech_recognition_handler(
    bot: Bot,
    msg: Message,
    config: &Config,
) -> Result<(), MyError> {
    let Some(message) = msg.reply_to_message() else {
        bot.send_message(msg.chat.id, "Ответьте на голосовое сообщение.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;

        return Ok(());
    };

    transcription_handler(bot, message, config).await?;

    Ok(())
}
