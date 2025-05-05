use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const DELETE_CALLBACK_DATA: &str = "delete_msg";

pub fn delete_message_button() -> InlineKeyboardMarkup {
    let delete_button = InlineKeyboardButton::callback("🗑️", DELETE_CALLBACK_DATA);

    InlineKeyboardMarkup::new(vec![vec![delete_button]])
}
