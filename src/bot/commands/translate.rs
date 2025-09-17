use crate::{
    bot::keyboards::{delete::delete_message_button, translate::create_language_keyboard},
    core::{
        config::Config,
        services::translation::{SUPPORTED_LANGUAGES, normalize_language_code},
    },
    errors::MyError,
};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, Message, ParseMode, ReplyParameters},
    utils::html::escape,
};
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
        target_lang = normalize_language_code(arg.trim());
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

    if text_to_translate.trim().is_empty() {
        bot.send_message(msg.chat.id, "В сообщении нет текста для перевода.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    if text_to_translate.trim().len() >= 3000 { // amm guys? I'm... I'm good at fixing issues... I did it because:
                                                // 1. Google Translator API has a limit of 3100+ characters
                                                // 2. I'm lazy to do paginator 🤙🤙🤙
        bot.send_message(msg.chat.id, "Текст превышает лимит в 3000 символов.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    if target_lang.is_empty() {
        bot.send_message(
            msg.chat.id,
            "Не удалось распознать язык. Пожалуйста, укажите корректный язык.",
        )
            .reply_parameters(ReplyParameters::new(msg.id))
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    let google_trans = GoogleTranslator::builder()
        .text_limit(12000usize)
        .delay(3usize)
        .timeout(50usize)
        .build();

    let res = google_trans
        .translate_async(text_to_translate, "", &target_lang)
        .await?;

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
