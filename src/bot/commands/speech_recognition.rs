use teloxide::dispatching::dialogue::GetChatId;
use crate::core::config::Config;
use crate::errors::MyError;
use crate::core::services::transcription::transcription_handler;
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;

pub async fn speech_recognition_handler(
    bot: Bot,
    msg: Message,
    config: &Config,
) -> Result<(), MyError> {
    if msg.reply_to_message().is_some() {
        transcription_handler(bot, msg.reply_to_message().unwrap().clone(), config).await?;
    } else {
        bot.send_message(msg.chat_id().unwrap(), "Ответьте на голосовое сообщение.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
    }
    Ok(())
}
