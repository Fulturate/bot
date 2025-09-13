use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const DELETE_CALLBACK_DATA: &str = "delete_msg";
pub const CONFIRM_DELETE_CALLBACK_DATA: &str = "delete_confirm";

pub(crate) fn delete_message_button(original_user_id: u64) -> InlineKeyboardMarkup {
    let callback_data = format!("{}:{}", DELETE_CALLBACK_DATA, original_user_id);
    let delete_button = InlineKeyboardButton::callback("🗑️", callback_data);
    InlineKeyboardMarkup::new(vec![vec![delete_button]])
}

pub(crate) fn confirm_delete_keyboard(original_user_id: u64) -> InlineKeyboardMarkup {
    let yes_callback = format!("{}:{}:yes", CONFIRM_DELETE_CALLBACK_DATA, original_user_id);
    let no_callback = format!("{}:{}:no", CONFIRM_DELETE_CALLBACK_DATA, original_user_id);

    let buttons = vec![
        InlineKeyboardButton::callback("Да", yes_callback),
        InlineKeyboardButton::callback("Нет", no_callback),
    ];

    InlineKeyboardMarkup::new(vec![buttons])
}