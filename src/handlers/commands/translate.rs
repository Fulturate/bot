use crate::config::Config;
use crate::util::{errors::MyError, transcription::transcription_handler};
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters, User};
use translators::{GoogleTranslator, Translator};

pub async fn translate_handler(
    bot: Bot,
    msg: Message,
    config: &Config,
    arg: String,
) -> Result<(), MyError> {
    let text_to_translate = msg.reply_to_message().unwrap().text().unwrap();
    let user = msg.from().unwrap();

    if msg.reply_to_message().is_some() {
        let target_lang = if arg.trim().is_empty() {
            get_user_target_language(config, user).await?
        } else {
            normalize_language_code(&arg.trim())
        };

        // let translated_text = translate_text(
        //     text_to_translate,
        //     &target_lang,
        //     translation_method,
        //     config
        // ).await?;

        let google_trans = GoogleTranslator::default();
        let res = google_trans
            .translate_async(text_to_translate, "", &*target_lang)
            .await
            .unwrap();

        let response = format!(
            "🌐 Перевод на {}:\n<blockquote>{}\n</blockquote>",
            target_lang.to_uppercase(),
            res,
        );

        bot.send_message(msg.chat.id, response)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        bot.send_message(msg.chat.id, "Нужно ответить на <b>то сообщение</b>, которое требуется перевести, чтобы использовать эту команду.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(ParseMode::Html)
            .await?;
    }
    Ok(())
}

async fn get_user_target_language(_config: &Config, user: &User) -> Result<String, MyError> {
    let lang_code = user.language_code.as_ref() // todo: add user settings
        .unwrap_or(&"en".to_string())
        .clone();

    Ok(normalize_language_code(&lang_code))
}

fn normalize_language_code(lang: &str) -> String {
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