use crate::core::config::Config;
use crate::core::db::schemas::user::User;
use crate::errors::MyError;
use crate::core::services::currency::converter::get_default_currencies;
use log::error;
use mongodb::bson::doc;
use oximod::Model;
use std::time::Instant;
use sysinfo::System;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters};

pub async fn start_handler(
    bot: Bot,
    message: Message,
    config: &Config,
    _arg: String,
) -> Result<(), MyError> {
    if message.chat.is_private() {
        let user = message.from.clone().unwrap();

        match User::find_one(doc! { "user_id": &user.id.to_string() }).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                let necessary_codes = get_default_currencies()?;

                return match User::new()
                    .user_id(user.id.to_string().clone())
                    .convertable_currencies(necessary_codes)
                    .save()
                    .await
                {
                    Ok(_) => {
                        bot.send_message(
                            message.chat.id,
                            "Welcome! You have been successfully registered",
                        )
                        .await?;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to save new user {} to DB: {}", &user.id, e);
                        bot.send_message(
                            message.chat.id,
                            "Something went wrong during registration. Please try again later.",
                        )
                        .await?;
                        Ok(())
                    }
                };
            }
            Err(e) => {
                error!("Database error while checking user {}: {}", &user.id, e);
                bot.send_message(
                    message.chat.id,
                    "A database error occurred. Please try again later.",
                )
                .await?;
                return Ok(());
            }
        };
    }
    let version = config.get_version();

    let start_time = Instant::now();
    bot.get_me().await?;
    let api_ping = start_time.elapsed().as_millis();

    let mut system_info = System::new_all();
    system_info.refresh_all();

    let total_ram_bytes = system_info.total_memory();
    let used_ram_bytes = system_info.used_memory();

    let total_ram_mb = total_ram_bytes / (1024 * 1024);
    let used_ram_mb = used_ram_bytes / (1024 * 1024);

    let cpu_usage_percent = system_info.global_cpu_usage();

    let response_message = format!(
        "<b>[BETA]</b> Telegram Bot by @Weever && @nixxoq\n\
        <pre>\
        > <b>Version</b>: {}\n\
        > <b>API Ping</b>: {} ms\n\
        > <b>CPU Usage</b>: {:.2}%\n\
        > <b>RAM Usage</b>: {}/{} MB\n\
        </pre>",
        version, api_ping, cpu_usage_percent, used_ram_mb, total_ram_mb
    );

    bot.send_message(message.chat.id, response_message)
        .reply_parameters(ReplyParameters::new(message.id))
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
