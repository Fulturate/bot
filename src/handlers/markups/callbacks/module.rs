use oximod::Model;
use teloxide::Bot;
use teloxide::payloads::{AnswerCallbackQuerySetters, EditMessageTextSetters};
use teloxide::prelude::{CallbackQuery, Requester};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::db::schemas::settings::Settings;
use crate::db::schemas::SettingsRepo;
use crate::handlers::commands::settings::{update_settings_message};
use crate::util::errors::MyError;

pub async fn module_select_handler(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), MyError> {
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

    let toggle_label = if module.enabled { "Выключить" } else { "Включить" };
    let toggle_cb = format!("module_toggle:{owner_type}:{owner_id}:{module_key}");

    let mut keyboard_rows: Vec<Vec<InlineKeyboardButton>> = vec![];

    keyboard_rows.push(vec![InlineKeyboardButton::callback(toggle_label, toggle_cb)]);

    for opt in module.options.iter() {
        let label = format!("{}: {}", opt.key, opt.value);
        let cb = format!("module_opt:{owner_type}:{owner_id}:{module_key}:{}", opt.key);
        keyboard_rows.push(vec![InlineKeyboardButton::callback(label, cb)]);
    }

    let back_button_cb = format!("settings_back:{owner_type}:{owner_id}");
    keyboard_rows.push(vec![InlineKeyboardButton::callback("⬅️ Назад", back_button_cb)]);

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
    println!("module_toggle_handler");
    let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
    println!("1");
    let owner_type = parts[1];
    println!("2:{}", owner_type);
    let owner_id = parts[2];
    println!("3:{}", owner_id);
    let module_key = parts[3];
    println!("4:{}", module_key);

    let mut settings = Settings::get_or_create(owner_id, owner_type).await?;
    if let Some(m) = settings.modules_mut().iter_mut().find(|m| m.key == module_key) {
        println!("5");
        m.enabled = !m.enabled;
        println!("6");
    }
    println!("7");
    settings.save().await?; // FIXME: this fucking ahh code freezes or gives error but i'm lazy to fix it
    println!("8");

    update_settings_message(bot, q.message.unwrap().clone(), owner_id.to_string(), owner_type.to_string()).await
}

pub async fn module_option_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
    let owner_type = parts[1];
    let owner_id = parts[2];
    let module_key = parts[3];
    let option_key = parts[4];

    let settings = Settings::get_or_create(owner_id, owner_type).await?;
    let module = settings.modules.iter().find(|m| m.key == module_key).unwrap();
    let opt = module.options.iter().find(|o| o.key == option_key).unwrap();

    bot.answer_callback_query(q.id.clone())
        .text(format!("Текущая опция «{}»: {}", opt.key, opt.value))
        .show_alert(true)
        .await?;
    Ok(())
}

pub async fn settings_back_handler(bot: Bot, q: CallbackQuery) -> Result<(), MyError> {
    let parts: Vec<_> = q.data.as_ref().unwrap().split(':').collect();
    let owner_type = parts[1];
    let owner_id = parts[2];

    update_settings_message(bot, q.message.unwrap().clone(), owner_id.to_string(), owner_type.to_string()).await
}
