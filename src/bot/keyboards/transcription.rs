use crate::util::paginator::{FrameBuild, Paginator};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const TRANSCRIPTION_MODULE_KEY: &str = "speech";

pub fn create_transcription_keyboard(
    current_page: usize,
    total_pages: usize,
    user_id: u64,
) -> InlineKeyboardMarkup {
    let summary_button = InlineKeyboardButton::callback("✨", format!("summarize:{}", user_id));
    let delete_button = InlineKeyboardButton::callback("🗑️", format!("delete_msg:{}", user_id));

    Paginator::new(TRANSCRIPTION_MODULE_KEY, total_pages)
        .current_page(current_page)
        .add_bottom_row(vec![summary_button])
        .add_bottom_row(vec![delete_button])
        .build()
}

pub fn create_summary_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        "back_to_full",
    )]])
}

pub fn create_retry_keyboard(message_id: i32, user_id: u64, action_type: &str, attempt: u32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔁 Повторить попытку",
        format!("retry_speech:{}:{}:{}:{}", message_id, user_id, action_type, attempt),
    )]/*, delete_message_button(user_id).inline_keyboard.first().unwrap().to_vec()*/])
}