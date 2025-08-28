use crate::config::Config;
use crate::db::schemas::user::User;
use log::{debug, error};
use mongodb::bson::doc;
use oximod::Model;
use std::error as error_handler;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::AnswerInlineQuerySetters;
use teloxide::prelude::Requester;
use teloxide::types::{
    Chat, ChatId, ChatKind, ChatPrivate, InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery,
    InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText, Me,
    ParseMode,
};
use uuid::Uuid;

pub async fn handle_currency_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
    me: Me,
) -> Result<(), Box<dyn error_handler::Error + Send + Sync>> {
    let user_id_str = q.from.id.to_string();

    let user_exists = User::find_one(doc! { "user_id": &user_id_str })
        .await?
        .is_some();

    if !user_exists {
        debug!("User {} not found. Offering to register.", user_id_str);

        let start_url = format!("https://t.me/{}?start=inl", me.username());

        let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
            "▶️ Register".to_string(),
            start_url.parse()?,
        )]]);

        let article = InlineQueryResultArticle::new(
            "register_prompt",
            "You are not registered",
            InputMessageContent::Text(InputMessageContentText::new(
                "To use the bot, please start a conversation with it first.",
            )),
        )
        .description("Click here to start a chat with the bot and unlock all features.")
        .reply_markup(keyboard);

        if let Err(e) = bot
            .answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
            .cache_time(10)
            .await
        {
            error!("Failed to send 'register' inline prompt: {:?}", e);
        }

        return Ok(());
    }

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

            if let Err(e) = bot
                .answer_inline_query(q.id, vec![result])
                .cache_time(2)
                .await
            {
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
