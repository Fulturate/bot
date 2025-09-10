use crate::bot::keyboards::translate::create_language_keyboard;
use crate::core::config::Config;
use crate::core::services::translation::{SUPPORTED_LANGUAGES, normalize_language_code};
use crate::errors::MyError;
use crate::bot::keyboards::delete::delete_message_button;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, Message, ParseMode, ReplyParameters};
use teloxide::utils::html::escape;
use translators::{GoogleTranslator, Translator};

pub async fn translate_handler(
    bot: Bot,
    msg: &Message,
    config: &Config,
    arg: String,
) -> Result<(), MyError> {
    let replied_to_message = match msg.reply_to_message() {
        Some(message) => message,
        None => {
            bot.send_message(
                msg.chat.id,
                "Нужно ответить на <b>то сообщение</b>, которое требуется перевести, чтобы использовать эту команду.",
            )
                .reply_parameters(ReplyParameters::new(msg.id))
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        }
    };

    let text_to_translate = match replied_to_message.text() {
        Some(text) => text,
        None => {
            bot.send_message(msg.chat.id, "Отвечать нужно на сообщение <b>с текстом</b>.")
                .reply_parameters(ReplyParameters::new(msg.id))
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        }
    };

    let user = msg.from.clone().unwrap();
    if replied_to_message.clone().from.unwrap().is_bot {
        bot.send_message(msg.chat.id, "Отвечать нужно на сообщение от пользователя.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let target_lang: String;

    if !arg.trim().is_empty() {
        target_lang = normalize_language_code(&arg.trim());
    } else {
        let redis_key = format!("user_lang:{}", user.id);
        let redis_client = config.get_redis_client();
        let cached_lang: Option<String> = redis_client.get(&redis_key).await?;

        if let Some(lang) = cached_lang {
            target_lang = lang;
        } else {
            let keyboard = create_language_keyboard(0);
            bot.send_message(msg.chat.id, "Выберите язык для перевода:")
                .reply_markup(keyboard)
                .reply_parameters(ReplyParameters::new(replied_to_message.id))
                .await?;
            return Ok(());
        }
    }

    let google_trans = GoogleTranslator::default();
    let res = google_trans
        .translate_async(text_to_translate, "", &*target_lang)
        .await
        .unwrap();

    let response = format!("<blockquote>{}\n</blockquote>", escape(&res));

    let lang_display_name = SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == target_lang)
        .map(|(_, name)| *name)
        .unwrap_or(&target_lang);

    let switch_lang_button =
        InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs".to_string());

    let mut keyboard = delete_message_button(user.id.0);
    if let Some(first_row) = keyboard.inline_keyboard.get_mut(0) {
        first_row.insert(0, switch_lang_button);
    } else {
        keyboard.inline_keyboard.push(vec![switch_lang_button]);
    }

    bot.send_message(msg.chat.id, response)
        .reply_parameters(ReplyParameters::new(replied_to_message.id))
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
