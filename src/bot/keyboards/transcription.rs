use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn create_transcription_keyboard(
    page: usize,
    total_pages: usize,
    user_id: u64,
) -> InlineKeyboardMarkup {
    let mut row = vec![];

    if page > 1 {
        row.push(InlineKeyboardButton::callback(
            "⬅️",
            format!("paginate:{}:{}:{}", page - 1, total_pages, user_id),
        ));
    }

    row.push(InlineKeyboardButton::callback(
        format!("📄 {}/{}", page, total_pages),
        "noop",
    ));

    if page < total_pages {
        row.push(InlineKeyboardButton::callback(
            "➡️",
            format!("paginate:{}:{}:{}", page + 1, total_pages, user_id),
        ));
    }

    let summary_button = InlineKeyboardButton::callback("✨", "summarize");
    let delete_button =
        InlineKeyboardButton::callback("🗑️", format!("delete_msg:{}", user_id));

    if total_pages > 1 {
        InlineKeyboardMarkup::new(vec![row, vec![summary_button, delete_button]])
    } else {
        InlineKeyboardMarkup::new(vec![vec![summary_button, delete_button]])
    }
}

pub fn create_summary_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        "back_to_full",
    )]])
}