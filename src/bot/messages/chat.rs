use crate::{
    bot::modules::Owner,
    core::db::schemas::{group::Group, settings::Settings, user::User},
    errors::MyError,
};
use log::{info};
use mongodb::bson::doc;
use oximod::ModelTrait;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::Requester,
    types::{ChatMemberUpdated, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    Bot,
};

pub async fn handle_bot_added(bot: Bot, update: ChatMemberUpdated) -> Result<(), MyError> {
    let id = update.chat.id.to_string();

    if update.new_chat_member.is_banned() || update.new_chat_member.is_left() {
        info!("Bot was kicked/banned. Deleting all data for ID: {}", &id);

        let owner_type = if update.chat.is_private() { "user" } else { "group" };

        if owner_type == "user" {
            User::delete(doc! { "user_id": &id }).await.ok();
        } else {
            Group::delete(doc! { "group_id": &id }).await.ok();
        }

        Settings::delete(doc! { "owner_id": &id, "owner_type": owner_type })
            .await
            .ok();

        return Ok(());
    }

    info!("Bot added to chat. ID: {}", &id);

    let welcome_text = if update.chat.is_private() {
        "<b>Добро пожаловать в Fulturate!</b> 👋\n\n\
            Я готов помочь с различными задачами!\n\n\
            Вот краткий список моих возможностей:\n\
            - 📥 <b>Скачивание медиа</b>: из различных источников.\n\
            - 💱 <b>Конвертация валют</b>: актуальные курсы всегда под рукой.\n\
            - 🤫 <b>Система «шепота»</b>: для более приватного общения.\n\n\
            Чтобы настроить модули под себя и узнать больше, используйте команду /settings."
            .to_string()
    } else {
        "<b>Спасибо, что добавили Fulturate в ваш чат!</b> 🎉\n\n\
            Я многофункциональный бот, готовый помогать вашему чату!\n\n\
            Для полноценной работы мне необходимы <b>права администратора</b>. \
            Это позволит мне обрабатывать команды и эффективно взаимодействовать с участниками.\n\n\
            Чтобы настроить мои модули и возможности, один из администраторов чата может использовать команду /settings."
            .to_string()
    };

    let news_link_button =
        InlineKeyboardButton::url("Канал с новостями", "https://t.me/fulturate".parse().unwrap());
    let terms_of_use_link_button = InlineKeyboardButton::url(
        "Условия использования",
        "https://telegra.ph/Terms-Of-Use--Usloviya-ispolzovaniya-09-21"
            .parse()?,
    );

    bot.send_message(update.chat.id, welcome_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
            news_link_button,
            terms_of_use_link_button,
        ]]))
        .await?;

    if update.chat.is_private() {
        if User::find_one(doc! { "user_id": &id }).await?.is_none() {
            User::new().user_id(id.clone()).save().await?;
        }
        let owner = Owner {
            id,
            r#type: "user".to_string(),
        };
        Settings::get_or_create(&owner).await?;
    } else {
        if Group::find_one(doc! { "group_id": &id }).await?.is_none() {
            Group::new().group_id(id.clone()).save().await?;
        }
        let owner = Owner {
            id,
            r#type: "group".to_string(),
        };
        Settings::get_or_create(&owner).await?;
    }

    Ok(())
}