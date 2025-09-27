use crate::core::services::translation::{LANGUAGES_PER_PAGE, SUPPORTED_LANGUAGES};
use crate::util::paginator::{ItemsBuild, Paginator};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn create_language_keyboard(page: usize, user_id: u64) -> InlineKeyboardMarkup {
    Paginator::from("tr", SUPPORTED_LANGUAGES)
        .per_page(LANGUAGES_PER_PAGE)
        .columns(2)
        .current_page(page)
        .set_callback_formatter(|p| format!("tr_page:{}:{}", p, user_id))
        .build(|(code, name)| {
            InlineKeyboardButton::callback(
                name.to_string(),
                format!("tr_lang:{}:{}", code, user_id),
            )
        })
}
