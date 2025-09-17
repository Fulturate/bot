use crate::{
    bot::keyboards::{delete::delete_message_button, translate::create_language_keyboard},
    core::{
        config::Config,
        services::translation::{SUPPORTED_LANGUAGES, normalize_language_code},
    },
    errors::MyError,
    util::paginator::{FrameBuild, Paginator},
};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, Message, ParseMode, ReplyParameters},
    utils::html::escape,
};
use translators::{GoogleTranslator, Translator};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslationCache {
    pub(crate) pages: Vec<String>,
    pub(crate) user_id: u64,
    pub(crate) original_url: Option<String>,
    pub(crate) target_lang: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslateJob {
    pub text: String,
    pub user_id: u64,
}

pub fn split_text_tr(text: &str, chunk_size: usize) -> Vec<String> {
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(chunk_size);

    for paragraph in text.split("\n\n") {
        if current_chunk.len() + paragraph.len() + 2 > chunk_size && !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
            current_chunk.clear();
        }
        if paragraph.len() > chunk_size {
            for part in paragraph.chars().collect::<Vec<_>>().chunks(chunk_size) {
                chunks.push(part.iter().collect());
            }
        } else {
            current_chunk.push_str(paragraph);
            current_chunk.push_str("\n\n");
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    chunks
}

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
        target_lang = normalize_language_code(arg.trim());
    } else {
        let redis_key = format!("user_lang:{}", user.id);
        let redis_client = config.get_redis_client();
        let cached_lang: Option<String> = redis_client.get(&redis_key).await?;

        if let Some(lang) = cached_lang {
            target_lang = lang;
        } else {
            let job = TranslateJob {
                text: text_to_translate.to_string(),
                user_id: user.id.0,
            };

            config
                .get_redis_client()
                .set(&format!("translate_job:{}", user.id), &job, 600)
                .await?;

            let keyboard = create_language_keyboard(0);
            bot.send_message(msg.chat.id, "Выберите язык для перевода:")
                .reply_markup(keyboard)
                .reply_parameters(ReplyParameters::new(replied_to_message.id))
                .await?;

            return Ok(());
        }
    }

    let text_chunks = split_text_tr(text_to_translate, 2800);

    let google_trans = GoogleTranslator::default();
    let translation_futures = text_chunks
        .iter()
        .map(|chunk| google_trans.translate_async(chunk, "", &target_lang));

    let results = join_all(translation_futures).await;
    let translated_chunks: Vec<String> = results.into_iter().filter_map(Result::ok).collect();
    let full_translated_text = translated_chunks.join("\n\n");

    if full_translated_text.is_empty() {
        bot.send_message(msg.chat.id, "Не удалось перевести текст.")
            .await?;
        return Ok(());
    }

    let display_pages = split_text_tr(&full_translated_text, 4000);

    let lang_display_name = SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == target_lang)
        .map(|(_, name)| *name)
        .unwrap_or(&target_lang);

    if display_pages.len() <= 1 {
        let response = format!("<blockquote>{}</blockquote>", escape(&full_translated_text));

        let switch_lang_button =
            InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs");

        let mut keyboard = delete_message_button(user.id.0);
        match keyboard.inline_keyboard.get_mut(0) {
            Some(first_row) => {
                first_row.insert(0, switch_lang_button);
            }
            None => {
                keyboard.inline_keyboard.push(vec![switch_lang_button]);
            }
        }

        bot.send_message(msg.chat.id, response)
            .reply_parameters(ReplyParameters::new(replied_to_message.id))
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    } else {
        let translation_id = Uuid::new_v4().to_string();
        let redis_key = format!("translation:{}", translation_id);

        let cache_data = TranslationCache {
            pages: display_pages.clone(),
            user_id: user.id.0,
            original_url: None,
            target_lang: target_lang.to_string(),
        };
        config
            .get_redis_client()
            .set(&redis_key, &cache_data, 3600)
            .await?;

        let switch_lang_button =
            InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs");
        let delete_button = delete_message_button(user.id.0)
            .inline_keyboard
            .remove(0)
            .remove(0);

        let keyboard = Paginator::new("tr", display_pages.len())
            .current_page(0)
            .set_callback_formatter(move |page| format!("tr:page:{}:{}", translation_id, page))
            .add_bottom_row(vec![switch_lang_button, delete_button])
            .build();

        let response_text = format!("<blockquote>{}</blockquote>", escape(&display_pages[0]));
        bot.send_message(msg.chat.id, response_text)
            .reply_parameters(ReplyParameters::new(replied_to_message.id))
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
