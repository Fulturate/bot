use crate::core::services::translation::{LANGUAGES_PER_PAGE, SUPPORTED_LANGUAGES};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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
