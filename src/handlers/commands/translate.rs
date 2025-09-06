use crate::config::Config;
use crate::util::errors::MyError;
use crate::util::inline::delete_message_button;
use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode, ReplyParameters,
};
use translators::{GoogleTranslator, Translator};

pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("uk", "🇺🇦 Українська"),
    ("en", "🇬🇧 English"),
    ("us", "🇺🇸 English (US)"),
    ("ru", "🇷🇺 Русский"),
    ("de", "🇩🇪 Deutsch"),
    ("fr", "🇫🇷 Français"),
    ("es", "🇪🇸 Español"),
    ("it", "🇮🇹 Italiano"),
    ("zh", "🇨🇳 中文"),
    ("ja", "🇯🇵 日本語"),
    ("ko", "🇰🇷 한국어"),
    ("pl", "🇵🇱 Polski"),
    ("ar", "🇸🇦 العربية"),
    ("pt", "🇵🇹 Português"),
    ("tr", "🇹🇷 Türkçe"),
    ("nl", "🇳🇱 Nederlands"),
    ("sv", "🇸🇪 Svenska"),
    ("no", "🇳🇴 Norsk"),
    ("da", "🇩🇰 Dansk"),
    ("fi", "🇫🇮 Suomi"),
    ("el", "🇬🇷 Ελληνικά"),
    ("he", "🇮🇱 עברית"),
    ("hi", "🇮🇳 हिन्दी"),
    ("id", "🇮🇩 Indonesia"),
    ("vi", "🇻🇳 Tiếng Việt"),
    ("th", "🇹🇭 ภาษาไทย"),
    ("cs", "🇨🇿 Čeština"),
    ("hu", "🇭🇺 Magyar"),
    ("ro", "🇷🇴 Română"),
    ("bg", "🇧🇬 Български"),
    ("sr", "🇷🇸 Српски"),
    ("hr", "🇭🇷 Hrvatski"),
    ("sk", "🇸🇰 Slovenčina"),
    ("sl", "🇸🇮 Slovenščina"),
    ("lt", "🇱🇹 Lietuvių"),
    ("lv", "🇱🇻 Latviešu"),
    ("et", "🇪🇪 Eesti"),
];
pub const LANGUAGES_PER_PAGE: usize = 6;

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

    let response = format!("<blockquote>{}\n</blockquote>", res,);

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

pub fn create_language_keyboard(page: usize) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let start = page * LANGUAGES_PER_PAGE;
    let end = std::cmp::min(start + LANGUAGES_PER_PAGE, SUPPORTED_LANGUAGES.len());

    if start >= end {
        return InlineKeyboardMarkup::new(keyboard);
    }

    let page_languages = &SUPPORTED_LANGUAGES[start..end];

    for chunk in page_languages.chunks(2) {
        let row = chunk
            .iter()
            .map(|(code, name)| {
                InlineKeyboardButton::callback(name.to_string(), format!("tr_lang:{}", code))
            })
            .collect();
        keyboard.push(row);
    }

    let mut nav_row: Vec<InlineKeyboardButton> = Vec::new();
    if page > 0 {
        nav_row.push(InlineKeyboardButton::callback(
            "⬅️".to_string(),
            format!("tr_page:{}", page - 1),
        ));
    }
    if end < SUPPORTED_LANGUAGES.len() {
        nav_row.push(InlineKeyboardButton::callback(
            "➡️".to_string(),
            format!("tr_page:{}", page + 1),
        ));
    }

    if !nav_row.is_empty() {
        keyboard.push(nav_row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub fn normalize_language_code(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "ua" | "ukrainian" | "украинский" | "uk" => "uk".to_string(),
        "ru" | "russian" | "русский" => "ru".to_string(),
        "en" | "english" | "английский" => "en".to_string(),
        "de" | "german" | "немецкий" => "de".to_string(),
        "fr" | "french" | "французский" => "fr".to_string(),
        "es" | "spanish" | "испанский" => "es".to_string(),
        "it" | "italian" | "итальянский" => "it".to_string(),
        "zh" | "chinese" | "китайский" => "zh".to_string(),
        "ja" | "japanese" | "японский" => "ja".to_string(),
        _ => lang.to_lowercase(),
    }
}
