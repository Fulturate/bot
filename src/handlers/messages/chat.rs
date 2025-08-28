use crate::db::schemas::group::Group;
use crate::handlers::markups::currency_keyboard::get_all_currency_codes;
use crate::util::currency::converter::{CURRENCY_CONFIG_PATH, CurrencyStruct};
use crate::util::errors::MyError;
use log::{error, info};
use oximod::ModelTrait;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{ChatMemberUpdated, ParseMode};

pub async fn handle_bot_added(bot: Bot, update: ChatMemberUpdated) -> Result<(), MyError> {
    let id = update.chat.id.to_string();
    info!("New chat added. ID: {}", id);
    
    let all_codes = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap())?;

    let necessary_codes = all_codes
        .iter()
        .filter(|c| {
            ["uah", "rub", "usd", "byn", "eur", "ton"].contains(&c.code.to_lowercase().as_str())
        })
        .cloned()
        .collect::<Vec<CurrencyStruct>>();

    let new_group = Group::new()
        .group_id(id.clone())
        .convertable_currencies(necessary_codes)
        .save()
        .await;

    if let Err(e) = new_group {
        error!("Could not save new group. Group id: {} | Error: {}", &id, e);
    }

    // todo: welcome message
    bot.send_message(update.chat.id, "Hello world".to_string())
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
