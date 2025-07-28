use crate::config::Config;
use crate::handlers::messages::sounder::sound_handlers;
use crate::util::errors::MyError;
use crate::util::inline::delete_message_button;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Message;
use teloxide::requests::Requester;
use teloxide::types::{ParseMode, ReplyParameters};
use tokio::task;

pub(crate) async fn messages_handlers(bot: Bot, message: Message) -> Result<(), MyError> {
    let config = Config::new().await;

    let bot_clone = bot.clone();
    let message_clone = message.clone();
    let config_clone = config.clone();

    task::spawn(async move {
        let bot = bot_clone;
        let message = message_clone;
        let config = config_clone;
        let original_user_id = message.from.clone().unwrap().id;

        if message.voice().is_some() || message.video_note().is_some() || message.audio().is_some()
        {
            if let Err(e) = sound_handlers(bot, message, &config).await {
                eprintln!("Sound handler failed: {:?}", e);
            }
        } else if let Some(text) = message.text() {
            let converter = config.get_currency_converter();
            let text_to_process = text;

            match converter.process_text(text_to_process).await {
                Ok(mut results) => {
                    if !results.is_empty() {
                        if results.len() > 5 {
                            results.truncate(5);
                        }
                        let formatted_blocks: Vec<String> = results
                            .into_iter()
                            .map(|result_block| {
                                let escaped_block = teloxide::utils::html::escape(&result_block);
                                format!("<blockquote expandable>{}</blockquote>", escaped_block)
                            })
                            .collect();

                        let final_message = formatted_blocks.join("\n");

                        if let Err(e) = bot
                            .send_message(message.chat.id, final_message)
                            .parse_mode(ParseMode::Html)
                            .reply_markup(delete_message_button(original_user_id.0))
                            .reply_parameters(ReplyParameters::new(message.id))
                            .await
                        {
                            eprintln!("Failed to send currency conversion result: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Currency conversion processing error: {:?}", e);
                }
            }
        }
    });

    Ok(())
}
