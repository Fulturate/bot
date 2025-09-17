use crate::{
    bot::{
        keyboards::cobalt::{make_photo_pagination_keyboard, make_single_url_keyboard},
        modules::{Owner, cobalt::CobaltSettings},
    },
    core::{
        config::Config,
        db::schemas::settings::Settings,
        services::cobalt::{DownloadResult, resolve_download_url},
    },
    errors::MyError,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use teloxide::{
    Bot,
    prelude::*,
    types::{
        InlineQuery, InlineQueryResult, InlineQueryResultArticle, InlineQueryResultPhoto,
        InlineQueryResultVideo, InputMessageContent, InputMessageContentText,
    },
};

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?)://[^\s/$.?#].[^\s]*$").unwrap());

pub async fn is_query_url(inline_query: InlineQuery) -> bool {
    if !URL_REGEX.is_match(&inline_query.query) {
        return false;
    };

    let owner = Owner {
        id: inline_query.from.id.to_string(),
        r#type: "user".to_string(),
    };

    match Settings::get_module_settings::<CobaltSettings>(&owner, "cobalt").await {
        Ok(settings) => settings.enabled,
        Err(_) => false,
    }
}

fn build_results_from_media(
    original_url: &str,
    media: DownloadResult,
    url_hash: &str,
    user_id: u64,
) -> Vec<InlineQueryResult> {
    match media {
        DownloadResult::Video(video_url) => {
            if let Ok(url) = video_url.parse() {
                let url_kb = make_single_url_keyboard(original_url);

                let result = InlineQueryResultVideo::new(
                    format!("cobalt_video:{}", url_hash),
                    url,
                    "video/mp4".parse().unwrap(),
                    "https://i.imgur.com/D0A9Gxh.png".parse().unwrap(), /* preview */
                    "Скачать видео".to_string(),
                )
                .reply_markup(url_kb);
                vec![result.into()]
            } else {
                vec![
                    InlineQueryResultArticle::new(
                        format!("cobalt_video:{}", url_hash),
                        "Видео не найдено",
                        InputMessageContent::Text(InputMessageContentText::new(
                            "❌ не удалось получить видео",
                        )),
                    )
                    .into(),
                ]
            }
        }
        DownloadResult::Photos { urls, .. } => {
            let total = urls.len();
            urls.into_iter()
                .enumerate()
                .filter_map(|(i, url_str)| {
                    if let (Ok(photo_url), Ok(thumb_url)) = (url_str.parse(), url_str.parse()) {
                        let result_id = format!("{}_{}", url_hash, i);

                        let keyboard = if total > 1 {
                            make_photo_pagination_keyboard(
                                url_hash,
                                i,
                                total,
                                user_id,
                                original_url,
                            )
                        } else {
                            make_single_url_keyboard(original_url)
                        };

                        let photo_result =
                            InlineQueryResultPhoto::new(result_id, photo_url, thumb_url)
                                .reply_markup(keyboard);

                        Some(photo_result.into())
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

pub async fn handle_cobalt_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
) -> Result<(), MyError> {
    let url = q.query.trim();

    if !URL_REGEX.is_match(url) {
        return Ok(());
    }

    let user_id = q.from.id.0;
    let user_id_str = q.from.id.to_string();

    let owner = Owner {
        id: user_id_str,
        r#type: "user".to_string(),
    };

    let url_hash_digest = md5::compute(url);
    let url_hash = format!("{:x}", url_hash_digest);
    let cache_key = format!("cobalt_cache:{}", url_hash);

    let redis = config.get_redis_client();

    let results = if let Ok(Some(cached_result)) = redis.get::<DownloadResult>(&cache_key).await {
        build_results_from_media(url, cached_result, &url_hash, user_id)
    } else {
        let settings = Settings::get_module_settings::<CobaltSettings>(&owner, "cobalt").await?;

        let cobalt_client = config.get_cobalt_client();
        let result = resolve_download_url(url, &settings, cobalt_client).await;

        match result {
            Ok(Some(download_result)) => {
                if let Err(e) = redis.set(&cache_key, &download_result, 42 * 60 * 60).await {
                    log::error!("Failed to cache cobalt result: {}", e);
                }
                build_results_from_media(url, download_result, &url_hash, user_id)
            }
            _ => {
                let error_article = InlineQueryResultArticle::new(
                    "error",
                    "Error",
                    InputMessageContent::Text(InputMessageContentText::new(
                        "Failed to process link. Media not found or an error occurred.",
                    )),
                )
                .description("Could not fetch media. Please try again later.");
                vec![error_article.into()]
            }
        }
    };
    bot.answer_inline_query(q.id, results).cache_time(0).await?;
    Ok(())
}
