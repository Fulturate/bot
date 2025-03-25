use dotenv::dotenv;
use teloxide::prelude::*;

#[derive(Clone)]
pub struct Config {
    bot: Bot,
    owners: Vec<i64>,
}

impl Config {
    pub async fn new() -> Self {
        dotenv().ok();

        let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN expected");
        let bot = Bot::new(bot_token);

        let owners: Vec<i64> = std::env::var("OWNERS")
            .expect("OWNERS expected")
            .split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect();

        Config { bot, owners }
    }

    pub fn get_bot(&self) -> &Bot {
        &self.bot
    }

    pub fn get_owners(&self) -> &Vec<i64> {
        &self.owners
    }
}
