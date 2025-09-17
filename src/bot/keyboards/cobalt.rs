use crate::util::paginator::{FrameBuild, Paginator};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn make_single_url_keyboard(url: &str) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "URL",
        url.parse().unwrap(),
    )]])
}

pub fn make_photo_pagination_keyboard(
    url_hash: &str,
    current_index: usize,
    total_photos: usize,
    user_id: u64,
    original_url: &str,
) -> InlineKeyboardMarkup {
    let url_button_row = vec![InlineKeyboardButton::url(
        "URL",
        original_url.to_string().parse().unwrap(),
    )];

    Paginator::new("cobalt", total_photos)
        .current_page(current_index)
        .add_bottom_row(url_button_row)
        .set_callback_formatter(move |page| {
            format!("cobalt:{}:{}:{}:{}", user_id, page, total_photos, url_hash)
        })
        .build()
}
