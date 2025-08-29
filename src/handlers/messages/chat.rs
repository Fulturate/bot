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
use teloxide::types::{ChatMemberUpdated, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use crate::util::db::create_default_values;

pub async fn handle_bot_experience(bot: Bot, update: ChatMemberUpdated) -> Result<(), MyError> {
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

    let new_query = create_default_values(id.clone(), update.chat.is_private()).await;

    match new_query {
        Ok(_) => {
            // todo: finish welcome message
            let news_link_button = InlineKeyboardButton::url("Канал с новостями о Боте", "https://t.me/fulturate".parse().unwrap());
            let github_link_button = InlineKeyboardButton::url("Github", "https://github.com/Fulturate/bot".parse().unwrap());

            bot.send_message(update.chat.id, "Добро пожаловать в Fulturate!\n\n<i>Мы тут когда нибудь что-то точно сделаем...</i>".to_string())
                .parse_mode(ParseMode::Html)
                .reply_markup(InlineKeyboardMarkup::new(vec![vec![news_link_button, github_link_button]]))
                .await?;
        }
        Err(e) => {
            error!("Could not save new entity. ID: {} | Error: {}", &id, e);
        }
    }

    Ok(())
}
