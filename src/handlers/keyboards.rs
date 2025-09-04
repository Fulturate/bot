use crate::db::schemas::settings::ModuleOption;
use base64::{engine::general_purpose, Engine as _};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn make_single_download_keyboard(url: &str, format: &str) -> InlineKeyboardMarkup {
    let button_text = if format == "audio" {
        "Download as Audio"
    } else {
        "Download as Video"
    };
    let encoded_url = general_purpose::STANDARD.encode(url);
    let callback_data = format!("download:{}:{}", format, encoded_url);
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        button_text.to_string(),
        callback_data,
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
            "cobalt_page:{}:{}:{}:{}",
            user_id, url_hash, prev_index, total_photos
        );
        row.push(InlineKeyboardButton::callback("⬅️", cb_data));
    }

    row.push(InlineKeyboardButton::url("URL", original_url.to_string().parse().unwrap()));

    row.push(InlineKeyboardButton::callback(
        format!("{}/{}", current_index + 1, total_photos),
        "cobalt_page:noop:0:0:0",
    ));

    if current_index + 1 < total_photos {
        let next_index = current_index + 1;
        let cb_data = format!(
            "cobalt_page:{}:{}:{}:{}",
            user_id, url_hash, next_index, total_photos
        );
        row.push(InlineKeyboardButton::callback("➡️", cb_data));
    }

    InlineKeyboardMarkup::new(vec![row])
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