use dotenv::dotenv;
use teloxide::prelude::*;

pub struct Config {
    bot: Bot,
    admin_id: Vec<i64>,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let bot = Bot::new(bot_token);
        // bot.parse_mode(ParseMode::Html);

        let admin_id: Vec<i64> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        Config { bot, admin_id }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_admin_id(&self) -> &Vec<i64> {
        &self.admin_id
    }
}
