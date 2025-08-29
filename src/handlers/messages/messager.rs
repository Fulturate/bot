use crate::config::Config;
use crate::handlers::messages::sounder::sound_handlers;
use crate::util::errors::MyError;
use crate::util::inline::delete_message_button;
use log::error;
use mongodb::bson::doc;
use oximod::Model;
use teloxide::Bot;
use teloxide::payloads::{EditMessageLiveLocationSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{Message, ParseMode, ReplyParameters};
use tokio::task;
use crate::db::schemas::group::Group;
use crate::db::schemas::user::User;
use crate::util::currency::converter::get_default_currencies;
use crate::util::db::create_default_values;

pub(crate) async fn handle_speech(bot: Bot, message: Message) -> Result<(), MyError> {
    let config = Config::new().await;
    let user = message.from.clone().unwrap();

    task::spawn(async move {
        if message.forward_from_user().is_some_and(|orig| orig.is_bot) || user.is_bot {
            return;
        }

        if let Err(e) = sound_handlers(bot, message.clone(), &config).await {
            error!("Sound handler failed: {:?}", e);
        }
    });

    Ok(())
}

pub(crate) async fn handle_currency(bot: Bot, message: Message) -> Result<(), MyError> {
    let config = Config::new().await;

    let bot_clone = bot.clone();

    task::spawn(async move {
        let user = message.from.clone().unwrap();
        let id = message.chat.id.to_string();

        if message.forward_from_user().is_some_and(|orig| orig.is_bot)
            || user.is_bot
            || message.via_bot.is_some()
        {
            return;
        }

        if !message.chat.is_private() {
            if !Group::exists(doc! { "group_id": &id }).await.unwrap() {
                let _ = create_default_values(id.clone(), false).await;
            }
        }

        if !User::exists(doc! { "user_id": user.id.0.to_string() }).await.unwrap() {
            let _ = create_default_values(user.id.0.to_string(), true).await;
        }

        let converter = config.get_currency_converter();
        if let Some(text) = message.text() {
            match converter.process_text(text, &message.chat).await {
                Ok(mut results) => {
                    if results.is_empty() {
                        return;
                    }

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

                    if let Err(e) = bot_clone
                        .send_message(message.chat.id, final_message)
                        .parse_mode(ParseMode::Html)
                        .reply_markup(delete_message_button(user.id.0))
                        .reply_parameters(ReplyParameters::new(message.id))
                        .await
                    {
                        error!("Failed to send currency conversion result: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Currency conversion processing error: {:?}", e);
                }
            }
        }
    });

    Ok(())
}
