use crate::db::schemas::settings::ModuleOption;
use base64::{Engine as _, engine::general_purpose};
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
    let mut row = Vec::new();

    if current_index > 0 {
        let prev_index = current_index - 1;
        let cb_data = format!(
            "cobalt:{}:{}:{}:{}",
            user_id, prev_index, total_photos, url_hash
        );
        row.push(InlineKeyboardButton::callback("⬅️", cb_data));
    }

    row.push(InlineKeyboardButton::callback(
        format!("{}/{}", current_index + 1, total_photos),
        "cobalt:noop",
    ));

    if current_index + 1 < total_photos {
        let next_index = current_index + 1;
        let cb_data = format!(
            "cobalt:{}:{}:{}:{}",
            user_id, next_index, total_photos, url_hash
        );
        row.push(InlineKeyboardButton::callback("➡️", cb_data));
    }

    InlineKeyboardMarkup::new(vec![row]).append_row(vec![InlineKeyboardButton::url(
        "URL",
        original_url.to_string().parse().unwrap(),
    )])
}

pub fn make_option_selection_keyboard(
    owner_type: &str,
    owner_id: &str,
    module_key: &str,
    option: &ModuleOption,
) -> InlineKeyboardMarkup {
    let options: Vec<&str> = match (module_key, option.key.as_str()) {
        ("cobalt", "video_quality") => vec!["720", "1080", "1440", "max"],
        ("cobalt", "audio_format") => vec!["mp3", "best", "wav", "opus"],
        ("cobalt", "attribution") => vec!["true", "false"],
        _ => vec![],
    };
    let buttons = options.into_iter().map(|opt| {
        let display_text = match (option.key.as_str(), opt) {
            ("attribution", "true") => "On",
            ("attribution", "false") => "Off",
            _ => opt,
        };
        let display = if opt == option.value {
            format!("• {} •", display_text)
        } else {
            display_text.to_string()
        };
        let cb_data = format!(
            "settings_set:{}:{}:{}:{}:{}",
            owner_type, owner_id, module_key, option.key, opt
        );
        InlineKeyboardButton::callback(display, cb_data)
    });
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = buttons.map(|b| vec![b]).collect();
    let back_cb = format!("module_select:{}:{}:{}", owner_type, owner_id, module_key);
    keyboard.push(vec![InlineKeyboardButton::callback("⬅️ Back", back_cb)]);
    InlineKeyboardMarkup::new(keyboard)
}
