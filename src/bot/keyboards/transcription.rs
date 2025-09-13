use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const TRANSCRIPTION_MODULE_KEY: &str = "transcription";

pub fn create_transcription_keyboard(
    current_page: usize,
    total_pages: usize,
    user_id: u64,
) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let mut nav_row = Vec::new();

    if current_page > 0 {
        nav_row.push(InlineKeyboardButton::callback(
            "⬅️",
            format!("{}:page:{}", TRANSCRIPTION_MODULE_KEY, current_page - 1),
        ));
    }

    nav_row.push(InlineKeyboardButton::callback(
        format!("📄 {}/{}", current_page + 1, total_pages),
        "noop",
    ));

    if current_page + 1 < total_pages {
        nav_row.push(InlineKeyboardButton::callback(
            "➡️",
            format!("{}:page:{}", TRANSCRIPTION_MODULE_KEY, current_page + 1),
        ));
    }

    if total_pages > 1 {
        keyboard.push(nav_row);
    }

    let action_row = vec![
        InlineKeyboardButton::callback("✨", "summarize"),
        InlineKeyboardButton::callback("🗑️", format!("delete_msg:{}", user_id)),
    ];
    keyboard.push(action_row);

    InlineKeyboardMarkup::new(keyboard)
}

pub fn create_summary_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        "back_to_full",
    )]])
}