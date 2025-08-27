use crate::db::schemas::{BaseFunctions, CurrenciesFunctions};
use crate::{
    db::functions::get_or_create,
    db::schemas::group::Group,
    db::schemas::user::User,
    handlers::markups::currency_keyboard::get_all_currency_codes,
    util::{currency::converter::CURRENCY_CONFIG_PATH, errors::MyError},
};
use oximod::Model;
use std::collections::HashSet;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters};

pub async fn handle_currency_update<T: BaseFunctions + CurrenciesFunctions + Model>(
    bot: Bot,
    msg: Message,
    code: String,
) -> Result<(), MyError> {
    let code = code.to_uppercase();

    let all_codes = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap())?;
    if !all_codes.iter().any(|c| c.code == code) {
        let message = format!("Currency code <code>{}</code> does not exist.", code);
        bot.send_message(msg.chat.id, message)
            .parse_mode(ParseMode::Html)
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }

    let entity_id = msg.chat.id.to_string();
    let entity = match get_or_create::<T>(entity_id).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to get or create entity: {:?}", e);
            let message = "Error: Could not access settings. Try again".to_string();
            bot.send_message(msg.chat.id, message)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
            return Ok(());
        }
    };

    let is_enabled = entity.get_currencies().iter().any(|c| c.code == code);

    let (update_func, action_text) = if is_enabled {
        (T::remove_currency(entity.get_id(), &code), "removed")
    } else {
        let currency_to_add = all_codes.iter().find(|x| x.code == code).unwrap();
        (T::add_currency(entity.get_id(), currency_to_add), "added")
    };

    let message = match update_func.await {
        Ok(_) => {
            format!(
                "Successfully {} <code>{}</code> from currency conversion.",
                action_text, code
            )
        }
        Err(e) => {
            eprintln!("--- Update failed: {:?} ---", e);
            "Failed to apply changes.".to_string()
        }
    };

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}

pub async fn currency_codes_handler(bot: Bot, msg: Message, code: String) -> Result<(), MyError> {
    if msg.chat.is_private() {
        handle_currency_update::<User>(bot, msg, code).await
    } else {
        handle_currency_update::<Group>(bot, msg, code).await
    }
}

async fn get_enabled_codes(msg: &Message) -> HashSet<String> {
    let chat_id = msg.chat.id.to_string();
    if msg.chat.is_private() {
        if let Ok(Some(user)) = <User as BaseFunctions>::find_by_id(chat_id).await {
            return user
                .get_currencies()
                .iter()
                .map(|c| c.code.clone())
                .collect();
        }
    } else {
        if let Ok(Some(group)) = <Group as BaseFunctions>::find_by_id(chat_id).await {
            return group
                .get_currencies()
                .iter()
                .map(|c| c.code.clone())
                .collect();
        }
    }
    HashSet::new()
}

pub async fn currency_codes_list_handler(bot: Bot, msg: Message) -> Result<(), MyError> {
    let mut message = String::from("Could not load available currencies.");

    if let Ok(codes) = get_all_currency_codes(CURRENCY_CONFIG_PATH.parse().unwrap()) {
        let enabled_codes = get_enabled_codes(&msg).await;

        let codes_list = codes
            .iter()
            .map(|currency| {
                let icon = if enabled_codes.contains(&currency.code) {
                    "✅"
                } else {
                    "❌"
                };
                format!("{} {} - {}", currency.flag, currency.code, icon)
            })
            .collect::<Vec<String>>()
            .join("\n");

        message = format!(
            // FIXME: is there any way to get all bot commands and then type /command_name without hardcoding?
            "Available currencies to set up: <blockquote expandable>{}</blockquote>\n\nUsage: <code>/setcurrency CURRENCY_CODE</code> (e.g., <code>/setcurrency UAH</code>) to enable/disable it.\n\nNotes:\n✅ - enabled\n❌ - disabled",
            codes_list
        );
    }

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}
