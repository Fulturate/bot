use crate::config::Config;
use ccobalt::model::request::{DownloadRequest, VideoQuality};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::{Request, Requester};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
    InlineQueryResultArticle, InputMessageContent,
    InputMessageContentText,
};
use crate::util::errors::MyError;

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?)://[^\s/$.?#].[^\s]*$").unwrap());

fn is_url(text: &str) -> bool {
    URL_REGEX.is_match(text)
}

pub async fn is_query_url(inline_query: InlineQuery) -> bool {
    URL_REGEX.is_match(&inline_query.query)
}

pub async fn handle_cobalt_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
) -> Result<(), MyError> {
    let url = &q.query;
    if is_url(url) {
        let request = DownloadRequest {
            url: url.to_string(),
            video_quality: Some(VideoQuality::Q720),
            ..Default::default()
        };

        let cobalt_client = config.get_cobalt_client();

        match cobalt_client.download_and_save(&request, "test", ".").await {
            Ok(_video_file) => {
                let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
                    "🔗 Open Link".to_string(),
                    url.parse()?,
                )]]);

                let article = InlineQueryResultArticle::new(
                    "cobalt_prompt",
                    "Cobalt Test",
                    InputMessageContent::Text(InputMessageContentText::new("Test")),
                )
                .description("Teestttt 22")
                .reply_markup(keyboard);

                bot.answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
                    .send()
                    .await?;

                Ok(())
            }
            Err(e) => {
                log::error!("Failed to download: {:?}", e);
                Ok(())
            }
        }
    } else {
        Ok(())
    }
}
