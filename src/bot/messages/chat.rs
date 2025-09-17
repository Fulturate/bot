use crate::{
    bot::modules::Owner,
    core::db::schemas::{group::Group, settings::Settings, user::User},
    errors::MyError,
};
use log::{error, info};
use mongodb::bson::doc;
use oximod::ModelTrait;
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatMemberUpdated, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
};

pub async fn handle_bot_added(bot: Bot, update: ChatMemberUpdated) -> Result<(), MyError> {
    let id = update.chat.id.to_string();
    let news_link_button = InlineKeyboardButton::url(
        "Канал с новостями о Боте",
        "https://t.me/fulturate".parse().unwrap(),
    );
    let github_link_button = InlineKeyboardButton::url(
        "Github",
        "https://github.com/Fulturate/bot".parse().unwrap(),
    );
    let msg = bot
        .send_message(
            update.chat.id,
            "Добро пожаловать в Fulturate!\n\n<i>Мы тут когда нибудь что-то точно сделаем...</i>"
                .to_string(),
        )
        .parse_mode(ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
            news_link_button,
            github_link_button,
        ]]));

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

    if update.chat.is_private() {
        if User::find_one(doc! { "user_id": &id }).await?.is_none() {
            User::new().user_id(id.clone()).save().await?;
            let owner = Owner {
                id,
                r#type: "user".to_string(),
            };
            Settings::create_with_defaults(&owner).await?;

            msg.await?;
        }
    } else if Group::find_one(doc! { "group_id": &id }).await?.is_none() {
        Group::new().group_id(id.clone()).save().await?;
        let owner = Owner {
            id,
            r#type: "group".to_string(),
        };
        Settings::create_with_defaults(&owner).await?;
        msg.await?;
    }

    Ok(())
}
