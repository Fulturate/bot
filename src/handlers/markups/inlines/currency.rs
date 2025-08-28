use crate::config::Config;
use log::{debug, error};
use std::error as error_handler;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::Requester;
use teloxide::types::{
    Chat, ChatId, ChatKind, ChatPrivate, InlineQuery, InlineQueryResult, InlineQueryResultArticle,
    InputMessageContent, InputMessageContentText, ParseMode,
};
use uuid::Uuid;

pub async fn handle_currency_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
) -> Result<(), Box<dyn error_handler::Error + Send + Sync>> {
    debug!("Handling currency inline query: {}", &q.query);

    let converter = config.get_currency_converter();
    let text_to_process = &q.query;

    // HACK
    let pseudo_chat = Chat {
        id: ChatId(q.from.id.0 as i64),
        kind: ChatKind::Private(ChatPrivate {
            first_name: Option::from(q.from.first_name.clone()),
            last_name: q.from.last_name.clone(),
            username: q.from.username.clone(),
        }),
    };

    match converter.process_text(text_to_process, &pseudo_chat).await {
        Ok(mut results) => {
            if results.is_empty() {
                debug!("No currency conversion results for: {}", &q.query);
                return Ok(());
            }

            if results.len() > 5 {
                results.truncate(5);
            }

            let raw_results = results.join("\n");

            let formatted = results
                .into_iter()
                .map(|result_block| {
                    let escaped_block = teloxide::utils::html::escape(&result_block);
                    format!("<blockquote expandable>{}</blockquote>", escaped_block)
                })
                .collect::<Vec<String>>();
            let final_message = formatted.join("\n");

            let article = InlineQueryResultArticle::new(
                Uuid::new_v4().to_string(),
                "Currency Conversion",
                InputMessageContent::Text(
                    InputMessageContentText::new(final_message.clone()).parse_mode(ParseMode::Html),
                ),
            )
            .description(raw_results);

            let result = InlineQueryResult::Article(article);

            if let Err(e) = bot.answer_inline_query(q.id, vec![result]).await {
                error!("Failed to answer currency inline query: {:?}", e);
            }
        }
        Err(e) => {
            error!(
                "Currency conversion processing error in inline mode: {:?}",
                e
            );
        }
    }

    Ok(())
}
