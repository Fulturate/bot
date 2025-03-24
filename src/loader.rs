use crate::config::Config;
use crate::handlers::commander::{command_handlers, Command};
use crate::handlers::messages::messager::messages_handlers;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::Requester;
use teloxide::types::Update;
use teloxide::utils::command::BotCommands;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub async fn run() -> Result<(), Error> {
    let config = Config::new().await;

    let command_menu = Command::bot_commands();
    config
        .get_bot()
        .set_my_commands(command_menu.clone())
        .await?;

    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(command_handlers);

    let message_handler = Update::filter_message().endpoint(messages_handlers);

    let handlers = dptree::entry()
        .branch(command_handler)
        .branch(message_handler);

    Dispatcher::builder(config.get_bot().clone(), handlers)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
