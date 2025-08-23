use crate::{
    config::Config,
    handlers::{
        commander::command_handlers, markups::markuper::callback_query_handlers,
        messages::messager::messages_handlers,
    },
    util::{enums::Command, errors::MyError},
};
use oximod::set_global_client;
use teloxide::{
    dispatching::{Dispatcher, HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Requester,
    types::Update,
    utils::command::BotCommands,
};

async fn run_bot(config: &Config) -> Result<(), MyError> {
    let command_menu = Command::bot_commands();
    config
        .get_bot()
        .set_my_commands(command_menu.clone())
        .await?;

    let commands = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(command_handlers);

    let messages = Update::filter_message().endpoint(messages_handlers);
    let callback_queries = Update::filter_callback_query().endpoint(callback_query_handlers);

    let handlers = dptree::entry()
        .branch(commands)
        .branch(callback_queries)
        .branch(messages);

    let bot_name = config.get_bot().get_my_name().await?;

    println!("Bot name: {:?}", bot_name.name);

    Dispatcher::builder(config.get_bot().clone(), handlers)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn run_database(config: &Config) -> Result<(), MyError> {
    let url = config.get_mongodb_url().to_owned();
    set_global_client(url).await?;

    Ok(())
}

pub async fn run() -> Result<(), MyError> {
    let config = Config::new().await;

    let _th = tokio::join!(run_database(&config), run_bot(&config));
    Ok(())
}
