use crate::bot::modules::registry::MOD_MANAGER;
use crate::bot::modules::Owner;
use crate::core::db::schemas::settings::Settings;
use crate::{
    bot::{
        callbacks::callback_query_handlers,
        commander::command_handlers,
        inlines::{
            cobalter::{handle_cobalt_inline, handle_inline_video, is_query_url},
            currency::{handle_currency_inline, is_currency_query},
            whisper::{handle_whisper_inline, is_whisper_query},
        },
        keyboards::delete::delete_message_button,
        messager::{handle_currency, handle_speech},
        messages::chat::handle_bot_added,
    },
    core::{config::Config,
           db::schemas::user::User as DBUser},
    errors::MyError,
    util::enums::Command,
};
use log::{debug, error, info};
use mongodb::bson::doc;
use oximod::{set_global_client, Model};
use serde::Deserialize;
use std::{convert::Infallible, fmt::Write, ops::ControlFlow, sync::Arc};
use teloxide::{
    dispatching::{
        Dispatcher, DpHandlerDescription, HandlerExt, MessageFilterExt, UpdateFilterExt,
    },
    dptree,
    error_handlers::LoggingErrorHandler,
    payloads::{AnswerInlineQuerySetters, SendDocumentSetters},
    prelude::{ChatId, Handler, Message, Requester},
    types::{
        Chat, InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
        InlineQueryResultArticle, InputFile, InputMessageContent, InputMessageContentText, Me,
        MessageId, ParseMode, ThreadId, Update, User,
    },
    update_listeners::Polling,
    utils::{command::BotCommands, html},
    Bot,
};

async fn root_handler(
    update: Update,
    config: Arc<Config>,
    bot: Bot,
    logic: Arc<Handler<'static, Result<(), MyError>, DpHandlerDescription>>,
    me: Me,
) -> Result<(), Infallible> {
    let deps = dptree::deps![update.clone(), config.clone(), bot.clone(), me.clone()];
    let result = logic.dispatch(deps).await;

    if let ControlFlow::Break(Err(err)) = result {
        let error_handler_endpoint: Handler<'static, (), DpHandlerDescription> =
            dptree::endpoint(handle_error);
        let error_deps = dptree::deps![Arc::new(err), update, config, bot];
        let _ = error_handler_endpoint.dispatch(error_deps).await;
    }

    Ok(())
}

async fn is_user_registered(q: InlineQuery) -> bool {
    let user_id_str = q.from.id.to_string();
    DBUser::find_one(doc! { "user_id": &user_id_str })
        .await
        .map_or(false, |user| user.is_some())
}

async fn prompt_registration(bot: Bot, q: InlineQuery, me: Me) -> Result<(), MyError> {
    let user_id_str = q.from.id.to_string();
    debug!("User {} not found. Offering to register.", user_id_str);

    let start_url = format!("https://t.me/{}?start=inl", me.username());

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "▶️ Зарегистрироваться".to_string(),
        start_url.parse()?,
    )]]);

    let article = InlineQueryResultArticle::new(
        "register_prompt",
        "Вы не зарегистрированы",
        InputMessageContent::Text(InputMessageContentText::new(
            "Чтобы использовать бота, пожалуйста, сначала начните диалог с ним.",
        )),
    )
        .description("Нажмите здесь, чтобы начать чат с ботом и разблокировать все функции.")
        .reply_markup(keyboard);

    if let Err(e) = bot
        .answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
        .cache_time(10)
        .await
    {
        error!("Failed to send 'register' inline prompt: {:?}", e);
    }

    Ok(())
}

#[derive(Deserialize)]
struct EnabledCheck {
    enabled: bool,
}

async fn are_any_inline_modules_enabled(q: InlineQuery) -> bool {
    let owner = Owner {
        id: q.from.id.to_string(),
        r#type: "user".to_string(),
    };

    if let Ok(settings) = Settings::get_or_create(&owner).await {
        for module in MOD_MANAGER.get_all_modules() {
            if module.is_enabled(&owner).await {
                if let Some(settings_json) = settings.modules.get(module.key()) {
                    if let Ok(check) = serde_json::from_value::<EnabledCheck>(settings_json.clone())
                    {
                        if check.enabled {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

async fn send_modules_disabled_message(bot: Bot, q: InlineQuery) -> Result<(), MyError> {
    let article = InlineQueryResultArticle::new(
        "modules_disabled",
        "Все модули выключены",
        InputMessageContent::Text(InputMessageContentText::new(
            "Все инлайн-модули выключены. Чтобы ими воспользоваться, активируйте их в настройках.",
        )),
    )
        .description("Используйте /settings в чате с ботом, чтобы включить их.");

    bot.answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
        .cache_time(10)
        .await?;
    Ok(())
}

pub fn inline_query_handler() -> Handler<'static, Result<(), MyError>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            dptree::filter_async(|q: InlineQuery| async move { !is_user_registered(q).await })
                .endpoint(prompt_registration),
        )
        .branch(
            dptree::filter_async(is_user_registered)
                .filter_async(|q: InlineQuery| async move { !are_any_inline_modules_enabled(q).await })
                .endpoint(send_modules_disabled_message),
        )
        .branch(
            dptree::filter_async(is_user_registered)
                .filter_async(are_any_inline_modules_enabled)
                .branch(dptree::filter_async(is_currency_query).endpoint(handle_currency_inline))
                .branch(dptree::filter_async(is_query_url).endpoint(handle_cobalt_inline))
                .branch(dptree::filter_async(is_whisper_query).endpoint(handle_whisper_inline)),
        )
}

async fn run_bot(config: Arc<Config>) -> Result<(), MyError> {
    let command_menu = Command::bot_commands();
    let bot = config.get_bot();
    bot.set_my_commands(command_menu.clone()).await?;

    let logic_handlers = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command_handlers),
        )
        .branch(
            Update::filter_message()
                .branch(Message::filter_text().endpoint(handle_currency))
                .branch(Message::filter_video_note().endpoint(handle_speech))
                .branch(Message::filter_voice().endpoint(handle_speech)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_query_handlers))
        .branch(Update::filter_my_chat_member().endpoint(handle_bot_added))
        .branch(Update::filter_inline_query().branch(inline_query_handler()))
        .branch(Update::filter_chosen_inline_result().endpoint(handle_inline_video));

    let me = bot.get_me().await?;
    info!("Bot name: {:?}", me.username());

    let listener = Polling::builder(bot.clone()).drop_pending_updates().build();

    Dispatcher::builder(bot.clone(), dptree::endpoint(root_handler))
        .dependencies(dptree::deps![config.clone(), Arc::new(logic_handlers), me])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;

    Ok(())
}

async fn run_database(config: Arc<Config>) -> Result<(), MyError> {
    let url = config.get_mongodb_url().to_owned();
    set_global_client(url.clone()).await?;
    info!("Database connected successfully. URL: {}", url);
    Ok(())
}

pub async fn run() -> Result<(), MyError> {
    let config = Arc::new(Config::new().await);
    let _th = tokio::join!(run_database(config.clone()), run_bot(config.clone()));
    Ok(())
}

fn extract_info(update: &Update) -> (Option<&User>, Option<&Chat>) {
    match &update.kind {
        teloxide::types::UpdateKind::Message(m) => (m.from.as_ref(), Some(&m.chat)),
        teloxide::types::UpdateKind::CallbackQuery(q) => {
            (Some(&q.from), q.message.as_ref().map(|m| m.chat()))
        }
        teloxide::types::UpdateKind::InlineQuery(q) => (Some(&q.from), None),
        teloxide::types::UpdateKind::MyChatMember(m) => (Some(&m.from), Some(&m.chat)),
        _ => (None, None),
    }
}

fn short_error_name(error: &MyError) -> String {
    format!("{}", error)
}

pub async fn handle_error(err: Arc<MyError>, update: Update, config: Arc<Config>, bot: Bot) {
    error!("An error has occurred: {:?}", err);

    let (user, chat) = extract_info(&update);
    let mut message_text = String::new();

    writeln!(&mut message_text, "🚨 <b>Новая ошибка!</b>\n").unwrap();

    if let Some(chat) = chat {
        let title = chat
            .title()
            .map_or("".to_string(), |t| format!(" ({})", html::escape(t)));
        writeln!(
            &mut message_text,
            "<b>В чате:</b> <code>{}</code>{}",
            chat.id, title
        )
            .unwrap();
    } else {
        writeln!(&mut message_text, "<b>В чате:</b> <i>(???)</i>").unwrap();
    }

    if let Some(user) = user {
        let username = user
            .username
            .as_ref()
            .map_or("".to_string(), |u| format!(" (@{})", u));
        let full_name = html::escape(&user.full_name());
        writeln!(
            &mut message_text,
            "<b>Вызвал:</b> {} (<code>{}</code>){}",
            full_name, user.id, username
        )
            .unwrap();
    } else {
        writeln!(&mut message_text, "<b>Вызвал:</b> <i>(???)</i>").unwrap();
    }

    let error_name = short_error_name(&err);
    writeln!(
        &mut message_text,
        "\n<b>Ошибка:</b>\n<blockquote expandable>{}</blockquote>",
        html::escape(&error_name)
    )
        .unwrap();

    let hashtag = "#error";
    writeln!(&mut message_text, "\n{}", hashtag).unwrap();

    let full_error_text = format!("{:#?}", err);
    let document = InputFile::memory(full_error_text.into_bytes()).file_name("error_details.txt");

    if let (Ok(log_chat_id), Ok(error_thread_id)) = (
        config.get_log_chat_id().parse::<i64>(),
        config.get_error_chat_thread_id().parse::<i32>(),
    ) {
        let chat_id = ChatId(log_chat_id);

        match bot
            .send_document(chat_id, document)
            .caption(message_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(delete_message_button(72))
            .message_thread_id(ThreadId(MessageId(error_thread_id)))
            .await
        {
            Ok(_) => info!("Error report sent successfully to chat {}", log_chat_id),
            Err(e) => error!("Failed to send error report to chat {}: {}", log_chat_id, e),
        }
    } else {
        error!(
            "LOG_CHAT_ID ({}) or ERROR_CHAT_THREAD_ID ({}) is not a valid integer",
            config.get_log_chat_id(),
            config.get_error_chat_thread_id()
        );
    }
}