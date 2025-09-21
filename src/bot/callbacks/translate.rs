use crate::{
    bot::{
        commands::translate::{TranslateJob, TranslationCache, split_text_tr},
        keyboards::{delete::delete_message_button, translate::create_language_keyboard},
    },
    core::{
        config::Config,
        services::translation::{SUPPORTED_LANGUAGES, normalize_language_code},
    },
    errors::MyError,
    util::paginator::{FrameBuild, Paginator},
};
use futures::future::join_all;
use teloxide::{
    ApiError, RequestError,
    prelude::*,
    types::{
        CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, MaybeInaccessibleMessage,
        Message, ParseMode,
    },
    utils::html::escape,
};
use translators::{GoogleTranslator, Translator};
use uuid::Uuid;

pub async fn handle_translate_callback(
    bot: Bot,
    q: CallbackQuery,
    config: &Config,
) -> Result<(), MyError> {
    if let (Some(data), Some(MaybeInaccessibleMessage::Regular(message))) = (&q.data, &q.message) {
        bot.answer_callback_query(q.id.clone()).await?;

        if let Some(rest) = data.strip_prefix("tr:page:") {
            let parts: Vec<_> = rest.split(':').collect();
            if parts.len() == 2 {
                let translation_id = parts[0];
                if let Ok(page) = parts[1].parse::<usize>() {
                    handle_translation_pagination(&bot, message, translation_id, page, config)
                        .await?;
                }
            }
        } else if data.starts_with("tr_page:") {
            handle_language_menu_pagination(bot, message, data).await?;
        } else if data.starts_with("tr_lang:") {
            handle_language_selection(bot, message, data, q.from.clone(), config).await?;
        } else if data == "tr_show_langs" {
            handle_show_languages(&bot, message, &q.from, config).await?;
        }
    } else {
        bot.answer_callback_query(q.id).await?;
    }
    Ok(())
}

async fn handle_translation_pagination(
    bot: &Bot,
    message: &Message,
    translation_id: &str,
    page: usize,
    config: &Config,
) -> Result<(), MyError> {
    let redis_key = format!("translation:{}", translation_id);
    let cache: Option<TranslationCache> = config.get_redis_client().get(&redis_key).await?;

    if let Some(cache_data) = cache {
        let lang_display_name = SUPPORTED_LANGUAGES
            .iter()
            .find(|(code, _)| *code == cache_data.target_lang)
            .map(|(_, name)| *name)
            .unwrap_or(&cache_data.target_lang);

        let switch_lang_button =
            InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs");

        let delete_button = delete_message_button(cache_data.user_id)
            .inline_keyboard
            .remove(0)
            .remove(0);

        let keyboard = Paginator::new("tr", cache_data.pages.len())
            .current_page(page)
            .set_callback_formatter(move |p| format!("tr:page:{}:{}", translation_id, p))
            .add_bottom_row(vec![switch_lang_button, delete_button])
            .build();

        let new_text = format!(
            "<blockquote>{}</blockquote>",
            escape(cache_data.pages.get(page).unwrap_or(&"".to_string()))
        );

        bot.edit_message_text(message.chat.id, message.id, new_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    } else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "Срок действия кеша перевода истек. Пожалуйста, переведите заново.",
        )
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![]]))
        .await?;
    }
    Ok(())
}

async fn handle_language_menu_pagination(
    bot: Bot,
    message: &Message,
    data: &str,
) -> Result<(), MyError> {
    if let Ok(page) = data.trim_start_matches("tr_page:").parse::<usize>() {
        let keyboard = create_language_keyboard(page);
        if let Err(e) = bot
            .edit_message_reply_markup(message.chat.id, message.id)
            .reply_markup(keyboard)
            .await
            && !matches!(e, RequestError::Api(ApiError::MessageNotModified))
        {
            return Err(MyError::from(e));
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
    let redis_client = config.get_redis_client();

    let redis_key_job = format!("translate_job:{}", user.id);
    let job: Option<TranslateJob> = redis_client.get_and_delete(&redis_key_job).await?;

    let Some(job) = job else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "Задача на перевод устарела. Пожалуйста, запросите перевод снова.",
        )
        .await?;
        return Ok(());
    };

    let text_to_translate = &job.text;

    let redis_key_user_lang = format!("user_lang:{}", user.id);
    redis_client
        .set(&redis_key_user_lang, &target_lang.to_string(), 7200)
        .await?;

    let normalized_lang = normalize_language_code(target_lang);

    let text_chunks = split_text_tr(text_to_translate, 2800);
    let google_trans = GoogleTranslator::default();
    let translation_futures = text_chunks
        .iter()
        .map(|chunk| google_trans.translate_async(chunk, "", &normalized_lang));

    let results = join_all(translation_futures).await;
    let translated_chunks: Vec<String> = results.into_iter().filter_map(Result::ok).collect();
    let full_translated_text = translated_chunks.join("\n\n");

    if full_translated_text.is_empty() {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "Не удалось перевести текст. Возможно, API временно недоступен.",
        )
        .await?;
        return Ok(());
    }

    let display_pages = split_text_tr(&full_translated_text, 4000);
    let lang_display_name = SUPPORTED_LANGUAGES
        .iter()
        .find(|(code, _)| *code == normalized_lang)
        .map(|(_, name)| *name)
        .unwrap_or(&normalized_lang);

    if display_pages.len() <= 1 {
        let response = format!("<blockquote>{}</blockquote>", escape(&full_translated_text));
        let switch_lang_button =
            InlineKeyboardButton::callback(lang_display_name.to_string(), "tr_show_langs");
        let mut keyboard = delete_message_button(user.id.0);
        if let Some(first_row) = keyboard.inline_keyboard.get_mut(0) {
            first_row.insert(0, switch_lang_button);
        } else {
            keyboard.inline_keyboard.push(vec![switch_lang_button]);
        }
        bot.edit_message_text(message.chat.id, message.id, response)
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
        redis_client.set(&redis_key, &cache_data, 3600).await?;

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
        bot.edit_message_text(message.chat.id, message.id, response_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

async fn handle_show_languages(
    bot: &Bot,
    message: &Message,
    user: &teloxide::types::User,
    config: &Config,
) -> Result<(), MyError> {
    if let Some(original_message) = message.reply_to_message() {
        if let Some(text) = original_message
            .text()
            .or_else(|| original_message.caption())
        {
            let job = TranslateJob {
                text: text.to_string(),
                user_id: user.id.0,
            };
            let redis_key_job = format!("translate_job:{}", user.id);
            config
                .get_redis_client()
                .set(&redis_key_job, &job, 600)
                .await?;
        }
    } else {
        bot.edit_message_text(
            message.chat.id,
            message.id,
            "Не удалось найти исходное сообщение для смены языка.",
        )
        .await?;
        return Ok(());
    }

    let keyboard = create_language_keyboard(0);
    bot.edit_message_text(
        message.chat.id,
        message.id,
        "Выберите новый язык для перевода:",
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}
