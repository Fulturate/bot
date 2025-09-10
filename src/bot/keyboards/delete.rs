use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const DELETE_CALLBACK_DATA: &str = "delete_msg";

pub(crate) fn delete_message_button(original_user_id: u64) -> InlineKeyboardMarkup {
    let callback_data = format!("{}:{}", DELETE_CALLBACK_DATA, original_user_id);
    let delete_button = InlineKeyboardButton::callback("🗑️", callback_data);

    InlineKeyboardMarkup::new(vec![vec![delete_button]])
}