use crate::handlers::markups::inlines::currency::handle_currency_inline;
use crate::handlers::messages::messager::{handle_currency, handle_speech};
use crate::util::currency::converter::is_currency_query;
use crate::{
    config::Config,
    handlers::{
        commander::command_handlers, markups::markuper::callback_query_handlers,
        messages::chat::handle_bot_added,
    },
    util::{enums::Command, errors::MyError},
};
use log::info;
use oximod::set_global_client;
use std::sync::Arc;
use teloxide::dispatching::MessageFilterExt;
use teloxide::prelude::{Handler, LoggingErrorHandler, Message};
use teloxide::update_listeners::Polling;
use teloxide::{
    dispatching::{Dispatcher, HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Requester,
    types::Update,
    utils::command::BotCommands,
};

pub fn inline_query_handler() -> Handler<
    'static,
    Result<(), MyError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::entry().branch(dptree::filter_async(is_currency_query).endpoint(handle_currency_inline))
        //.branch(dptree::filter_async(is_query_url).endpoint(handle_cobalt_inline))
}

async fn run_bot(config: Arc<Config>) -> Result<(), MyError> {
    let command_menu = Command::bot_commands();
    config
        .get_bot()
        .set_my_commands(command_menu.clone())
        .await?;

    let handlers = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(command_handlers),
        )
        .branch(
            Update::filter_message()
                .branch(Message::filter_text().endpoint(handle_currency))
                .branch(Message::filter_voice().endpoint(handle_speech)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_query_handlers))
        .branch(Update::filter_my_chat_member().endpoint(handle_bot_added))
        .branch(Update::filter_inline_query().branch(inline_query_handler()));

    let bot = config.get_bot().clone();
    let bot_name = config.get_bot().get_my_name().await?;

    info!("Bot name: {:?}", bot_name.name);
    let listener = Polling::builder(bot.clone()).drop_pending_updates().build();

    Dispatcher::builder(bot.clone(), handlers)
        .dependencies(dptree::deps![config.clone()])
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
