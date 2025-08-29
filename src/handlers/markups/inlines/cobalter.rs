use crate::config::Config;
use ccobalt::model::request::{DownloadRequest, VideoQuality};
use once_cell::sync::Lazy;
use regex::Regex;
use std::error as error_handler;
use std::str::FromStr;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::{AnswerInlineQuery, AnswerInlineQuerySetters, SendVideoSetters};
use teloxide::prelude::{Request, Requester};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
    InlineQueryResultArticle, InlineQueryResultsButton, InputMessageContent,
    InputMessageContentText,
};

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?)://[^\s/$.?#].[^\s]*$").unwrap());

#[allow(dead_code)]
fn is_url(text: &str) -> bool {
    URL_REGEX.is_match(text)
}

pub async fn is_query_url(inline_query: InlineQuery) -> bool {
    URL_REGEX.is_match(&inline_query.query)
}

fn url_to_reqwest_url(url: String) -> reqwest::Url {
    reqwest::Url::from_str(&url).unwrap()
}

pub async fn handle_cobalt_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
) -> Result<(), Box<dyn error_handler::Error + Send + Sync>> {
    let url = &q.query;
    if is_url(url) {
        let request = DownloadRequest {
            url: url.to_string(),
            video_quality: Some(VideoQuality::Q720),
            ..Default::default()
        };

        let cobalt_client = config.get_cobalt_client();

        match cobalt_client.download_and_save(&request, "test", ".").await {
            Ok(video_file) => {
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
        // Not a supported URL, optionally handle or ignore
        Ok(())
    }
}
