use crate::config::Config;
use crate::util::errors::MyError;
use std::time::Instant;
use sysinfo::System;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters};

pub async fn start_handler(bot: Bot, message: Message, config: &Config) -> Result<(), MyError> {
    let version = config.get_version();

    let start_time = Instant::now();
    bot.get_me().await?;
    let api_ping = start_time.elapsed().as_millis();

    let mut system_info = System::new_all();
    system_info.refresh_all();

    let total_ram_bytes = system_info.total_memory();
    let used_ram_bytes = system_info.used_memory();

    let total_ram_mb = (total_ram_bytes as f64 / (1024.0 * 1024.0)) as u64; // tried to use "as"
    let used_ram_mb = (used_ram_bytes as f64 / (1024.0 * 1024.0)) as u64;

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
