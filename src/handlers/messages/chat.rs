use crate::db::schemas::group::Group;
use crate::db::schemas::user::User;
use crate::util::currency::converter::get_default_currencies;
use crate::util::errors::MyError;
use log::{error, info};
use mongodb::bson::doc;
use oximod::ModelTrait;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatMemberUpdated, ParseMode};

pub async fn handle_bot_added(bot: Bot, update: ChatMemberUpdated) -> Result<(), MyError> {
    let id = update.chat.id.to_string();

    if update.new_chat_member.is_banned() || update.new_chat_member.is_left() {
        info!("Administrator is banned or user is blocked me. Deleting from DB");

        let delete_result = if update.chat.is_private() {
            User::delete(doc! { "user_id": id.clone() }).await
        } else {
            Group::delete(doc! { "group_id": id.clone() }).await
        };

        if let Err(e) = delete_result {
            error!("Could not delete entity. ID: {} | Error: {}", &id, e);
        }

        return Ok(());
    }

    info!("New chat added. ID: {}", id);

    let necessary_codes = get_default_currencies()?;

    let new_query = if update.chat.is_private() {
        User::new()
            .user_id(id.clone())
            .convertable_currencies(necessary_codes)
            .save()
            .await
    } else {
        Group::new()
            .group_id(id.clone())
            .convertable_currencies(necessary_codes)
            .save()
            .await
    };

    match new_query {
        Ok(_) => {
            // todo: welcome message
            bot.send_message(update.chat.id, "Hello world".to_string())
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            error!("Could not save new entity. ID: {} | Error: {}", &id, e);
        }
    }

    Ok(())
}
