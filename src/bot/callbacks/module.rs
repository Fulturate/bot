use crate::bot::commands::currency_settings::update_settings_message;
use crate::bot::keyboards::cobalt::make_option_selection_keyboard;
use crate::core::db::schemas::SettingsRepo;
use crate::core::db::schemas::settings::Settings;
use crate::errors::MyError;
use teloxide::Bot;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn module_select_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let data = match q.data.as_ref() {
        Some(d) => d,
        None => return Ok(()),
    };
    let message = match q.message.as_ref() {
        Some(m) => m,
        None => return Ok(()),
    };

    let parts: Vec<_> = data.split(':').collect();
    if parts.len() < 4 {
        log::error!("Invalid callback data format: {}", data);
        return Ok(());
    }
    let owner_type = parts[1];
    let owner_id = parts[2];
    let module_key = parts[3];

    let mut settings = Settings::get_or_create(owner_id, owner_type).await?;
    let module = settings
        .modules_mut()
        .iter_mut()
        .find(|m| m.key == module_key)
        .unwrap();

    let toggle_label = if module.enabled {
        "Выключить"
    } else {
        "Включить"
    };
    let toggle_cb = format!("module_toggle:{owner_type}:{owner_id}:{module_key}");

    let mut keyboard_rows: Vec<Vec<InlineKeyboardButton>> = vec![];

    keyboard_rows.push(vec![InlineKeyboardButton::callback(
        toggle_label,
        toggle_cb,
    )]);

    for opt in module.options.iter() {
        let label = format!("{}: {}", opt.key, opt.value);
        let cb = format!(
            "module_opt:{owner_type}:{owner_id}:{module_key}:{}",
            opt.key
        );
        keyboard_rows.push(vec![InlineKeyboardButton::callback(label, cb)]);
    }

    let back_button_cb = format!("settings_back:{owner_type}:{owner_id}");
    keyboard_rows.push(vec![InlineKeyboardButton::callback(
        "⬅️ Назад",
        back_button_cb,
    )]);

    let keyboard = InlineKeyboardMarkup::new(keyboard_rows);

    bot.edit_message_text(
        message.chat().id,
        message.id(),
        format!("⚙️ Настройки модуля: {}", module.description),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub async fn module_toggle_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
    let owner_type = parts[1];
    let owner_id = parts[2];
    let module_key = parts[3];

    Settings::update_module(owner_id, owner_type, module_key, |module| {
        module.enabled = !module.enabled;
    })
    .await?;

    update_settings_message(
        bot,
        q.message.unwrap().clone(),
        owner_id.to_string(),
        owner_type.to_string(),
    )
    .await
}

// pub async fn module_option_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
//     let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
//     let owner_type = parts[1];
//     let owner_id = parts[2];
//     let module_key = parts[3];
//     let option_key = parts[4];
//
//     let settings = Settings::get_or_create(owner_id, owner_type).await?;
//     let module = settings
//         .modules
//         .iter()
//         .find(|m| m.key == module_key)
//         .unwrap();
//     let opt = module.options.iter().find(|o| o.key == option_key).unwrap();
//
//     bot.answer_callback_query(q.id.clone())
//         .text(format!("Текущая опция «{}»: {}", opt.key, opt.value))
//         .show_alert(true)
//         .await?;
//     Ok(())
// }
pub async fn module_option_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let message = q
        .message
        .as_ref()
        .ok_or_else(|| MyError::Other("No message in callback".into()))?;
    let data = q
        .data
        .as_ref()
        .ok_or_else(|| MyError::Other("No data in callback".into()))?;

    let parts: Vec<_> = data.split(':').collect();
    let owner_type = parts[1];
    let owner_id = parts[2];
    let module_key = parts[3];
    let option_key = parts[4];

    let settings = Settings::get_or_create(owner_id, owner_type).await?;
    let module = settings
        .modules
        .iter()
        .find(|m| m.key == module_key)
        .unwrap();
    let opt = module.options.iter().find(|o| o.key == option_key).unwrap();

    let keyboard = make_option_selection_keyboard(owner_type, owner_id, module_key, opt);

    let option_name = option_key.replace('_', " ");
    bot.edit_message_text(
        message.chat().id,
        message.id(),
        format!("Select a value for '{}':", option_name),
    )
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub async fn settings_back_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
    let owner_type = parts[1];
    let owner_id = parts[2];

    update_settings_message(
        bot,
        q.message.unwrap().clone(),
        owner_id.to_string(),
        owner_type.to_string(),
    )
    .await
}

pub async fn settings_set_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    q.message
        .as_ref()
        .ok_or_else(|| MyError::Other("No message in callback".into()))?;

    let data = q
        .data
        .clone()
        .ok_or_else(|| MyError::Other("No data in callback".into()))?;

    let parts: Vec<_> = data.split(':').collect();
    if parts.len() < 6 {
        return Err(MyError::Other(
            "Invalid callback data for settings_set".into(),
        ));
    }
    let owner_type = parts[1];
    let owner_id = parts[2];
    let module_key = parts[3];
    let option_key = parts[4];
    let new_value = parts[5].to_string();

    Settings::update_module(owner_id, owner_type, module_key, |module| {
        if let Some(option) = module.options.iter_mut().find(|o| o.key == option_key) {
            option.value = new_value;
        }
    })
    .await?;

    let mut updated_q = q;
    updated_q.data = Some(format!(
        "module_select:{}:{}:{}",
        owner_type, owner_id, module_key
    ));

    module_select_handler(bot, updated_q).await
}
