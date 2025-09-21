use crate::{
    bot::modules::Owner,
    core::{
        config::Config,
        db::schemas::{settings::Settings, user::User},
    },
    errors::MyError,
};
use mongodb::bson::doc;
use oximod::Model;
use std::time::Instant;
use sysinfo::System;
use teloxide::{
    prelude::*,
    types::{ParseMode, ReplyParameters},
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn start_handler(
    bot: Bot,
    message: Message,
    config: &Config,
    _arg: String,
) -> Result<(), MyError> {
    let mut is_new_user = false;

    if message.chat.is_private() {
        if let Some(user) = message.from {
            if User::find_one(doc! { "user_id": &user.id.to_string() }).await?.is_none() {
                is_new_user = true;
                User::new().user_id(user.id.to_string().clone()).save().await?;

                let owner = Owner {
                    id: user.id.to_string(),
                    r#type: "user".to_string(),
                };
                Settings::create_with_defaults(&owner).await?;
            }
        }
    }

    let version = config.get_version();

    let start_time = Instant::now();
    bot.get_me().await?;
    let api_ping = start_time.elapsed().as_millis();

    let mut system_info = System::new_all();
    system_info.refresh_all();

    let total_ram_mb = system_info.total_memory() / (1024 * 1024);
    let used_ram_mb = system_info.used_memory() / (1024 * 1024);
    let cpu_usage_percent = system_info.global_cpu_usage();

    let welcome_part = if is_new_user {
        "<b>Добро пожаловать!</b> 👋\n\n\
            Я Fulturate — ваш многофункциональный ассистент. \
            Чтобы посмотреть все возможности и настроить меня, используйте команду /settings.\n\n".to_string()
    } else {
        "<b>Fulturate тут!</b> ⚙️\n\n".to_string()
    };

    let response_message = format!(
        "{welcome_part}\
        <b>Статус системы:</b>\n\
        <pre>\
        > Версия:      {}\n\
        > Пинг API:    {} мс\n\
        > Нагрузка ЦП: {:.2}%\n\
        > ОЗУ:         {}/{} МБ\n\
        </pre>",
        version, api_ping, cpu_usage_percent, used_ram_mb, total_ram_mb
    );

    let news_link_button =
        InlineKeyboardButton::url("Канал с новостями", "https://t.me/fulturate".parse().unwrap());
    let terms_of_use_link_button = InlineKeyboardButton::url(
        "Условия использования",
        "https://telegra.ph/Terms-Of-Use--Usloviya-ispolzovaniya-09-21"
            .parse()
            .unwrap(),
    );

    bot.send_message(message.chat.id, response_message)
        .reply_parameters(ReplyParameters::new(message.id))
        .parse_mode(ParseMode::Html)
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
            news_link_button,
            terms_of_use_link_button,
        ]]))
        .await?;

    Ok(())
}