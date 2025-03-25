use crate::config::Config;
use std::time::Instant;
use sysinfo::System;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use crate::util::errors::MyError;

pub async fn start_handler(bot: Bot, message: Message, _: &Config) -> Result<(), MyError> {
    let version = std::env::var("VERSION").unwrap_or_else(|_| "Unknown".to_string());

    let start_time = Instant::now();
    bot.get_me().await?;
    let api_ping = start_time.elapsed().as_millis();

    let mut system_info = System::new_all();
    system_info.refresh_all();
    let total_ram_mb = system_info.total_memory() / 1024 / 1024;
    let used_ram_mb = (system_info.total_memory() - system_info.free_memory()) / 1024 / 1024;
    let cpu_usage_percent = system_info.global_cpu_usage();

    let response_message = format!(
        "<b>[BETA]</b> Telegram Bot by @Weever\n\
        <pre>\
        > <b>Version</b>: {}\n\
        > <b>API Ping</b>: {} ms\n\
        > <b>CPU Usage</b>: {:.2}%\n\
        > <b>RAM Usage</b>: {}/{} MB\n\
        </pre>",
        version, api_ping, cpu_usage_percent, used_ram_mb, total_ram_mb
    );

    bot.send_message(message.chat.id, response_message)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}
