use crate::bot::handlers::commands::translate::{
    SUPPORTED_LANGUAGES, create_language_keyboard, normalize_language_code,
};
use crate::config::Config;
use crate::errors::MyError;
use crate::util::inline::delete_message_button;
use teloxide::Bot;
use teloxide::payloads::{EditMessageReplyMarkupSetters, EditMessageTextSetters};
use teloxide::prelude::Requester;
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, MaybeInaccessibleMessage, Message, ParseMode,
};
use teloxide::utils::html::escape;
use teloxide::{ApiError, RequestError};
use translators::{GoogleTranslator, Translator};

pub async fn handle_translate_callback(
    bot: Bot,
    q: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    let callback_id = q.id.clone();

    if let (Some(data), Some(MaybeInaccessibleMessage::Regular(message))) = (&q.data, &q.message) {
        bot.answer_callback_query(callback_id).await?;

        if data.starts_with("tr_page:") {
            handle_pagination(bot, message, data).await?;
        } else if data.starts_with("tr_lang:") {
            handle_language_selection(bot, message, data, q.from.clone(), config).await?;
        } else if data == "tr_show_langs" {
            handle_show_languages(bot, message).await?;
        }
    } else {
        bot.answer_callback_query(callback_id).await?;
    }
    Ok(())
}

async fn handle_pagination(bot: Bot, message: &Message, data: &str) -> Result<(), MyError> {
    match data.trim_start_matches("tr_page:").parse::<usize>() {
        Ok(page) => {
            let keyboard = create_language_keyboard(page);
            if let Err(e) = bot
                .edit_message_reply_markup(message.chat.id, message.id)
                .reply_markup(keyboard)
                .await
            {
                if let RequestError::Api(ApiError::MessageNotModified) = e {
                } else {
                    return Err(MyError::from(e));
                }
            }
        }
        Err(_) => {
            log::warn!("Failed to parse page number from: {}", data);
        }
    }
    Ok(())
}

async fn handle_language_selection(
    bot: Bot,
    message: &Message,
    data: &str,
    user: teloxide::types::User,
    config: &Config,
) -> Result<(), MyError> {
    let target_lang = data.trim_start_matches("tr_lang:");

    let original_message = match message.reply_to_message() {
        Some(msg) => msg,
        None => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "Ошибка: не удалось найти исходное сообщение. Попробуйте снова.",
            )
                .await?;
            return Ok(());
        }
    };

    let text_to_translate = match original_message.text() {
        Some(text) => text,
        None => {
            bot.edit_message_text(
                message.chat.id,
                message.id,
                "В исходном сообщении нет текста для перевода.",
            )
                .await?;
            return Ok(());
        }
    };

    let redis_key = format!("user_lang:{}", user.id);
    let redis_client = config.get_redis_client();
    let ttl_seconds = 2 * 60 * 60;
    redis_client
        .set(&redis_key, &target_lang.to_string(), ttl_seconds)
        .await?;

    let normalized_lang = normalize_language_code(target_lang);
    let google_trans = GoogleTranslator::default();
    let res = google_trans
        .translate_async(text_to_translate, "", &*normalized_lang)
        .await
        .unwrap();

    let response = format!("<blockquote>{}\n</blockquote>", escape(&res));

    let lang_display_name = SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == normalized_lang)
        .map(|(_, name)| *name)
        .unwrap_or(&normalized_lang);

    let switch_lang_button =
        InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs".to_string());

    let mut keyboard = delete_message_button(user.id.0);
    if let Some(first_row) = keyboard.inline_keyboard.get_mut(0) {
        first_row.insert(0, switch_lang_button);
    } else {
        keyboard.inline_keyboard.push(vec![switch_lang_button]);
    }

    if let Err(e) = bot
        .edit_message_text(message.chat.id, message.id, response)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await
    {
        if let RequestError::Api(ApiError::MessageNotModified) = e {
        } else {
            return Err(MyError::from(e));
        }
    }

    Ok(())
}

async fn handle_show_languages(bot: Bot, message: &Message) -> Result<(), MyError> {
    let keyboard = create_language_keyboard(0);
    if let Err(e) = bot
        .edit_message_text(message.chat.id, message.id, "Выберите язык для перевода:")
        .reply_markup(keyboard)
        .await
    {
        if let RequestError::Api(ApiError::MessageNotModified) = e {
        } else {
            return Err(MyError::from(e));
        }
    }
    Ok(())
}