use crate::config::Config;
use crate::db::schemas::settings::Settings;
use crate::db::schemas::SettingsRepo;
use crate::util::errors::MyError;
use ccobalt::model::request::{DownloadRequest, FilenameStyle, VideoQuality};
use ccobalt::model::response::DownloadResponse;
use mime::Mime;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{
    InlineQuery, InlineQueryResult, InlineQueryResultArticle, InlineQueryResultPhoto,
    InlineQueryResultVideo, InputMessageContent, InputMessageContentText,
};
use teloxide::Bot;

const FALLBACK_THUMB_URL: &str = "https://i.imgur.com/424242.png"; // todo: change it to a real thumbnail

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownloadResult {
    Video(String),
    Photos(Vec<String>),
}

pub async fn resolve_download_url(
    url: &str,
    settings: &Settings,
    client: &ccobalt::Client,
) -> Result<Option<DownloadResult>, MyError> {
    let get_opt = |key: &str| -> String {
        settings
            .modules
            .iter()
            .find(|m| m.key == "cobalt")
            .and_then(|m| m.options.iter().find(|o| o.key == key))
            .map(|o| o.value.clone())
            .unwrap_or_default()
    };

    let cobalt_req = DownloadRequest {
        url: url.to_string(),
        filename_style: Some(FilenameStyle::Pretty),
        video_quality: Some(match get_opt("video_quality").as_str() {
            "720" => VideoQuality::Q720,
            "1080" => VideoQuality::Q1080,
            "1440" => VideoQuality::Q1440,
            "max" => VideoQuality::Max,
            _ => VideoQuality::Q720,
        }),
        ..Default::default()
    };

    let response = client.resolve_download(&cobalt_req).await?;

    match response {
        DownloadResponse::Error { error } => {
            log::error!("Cobalt API error: {:?}", error);
            Err(error.into())
        }
        DownloadResponse::Picker { picker, .. } => {
            let photo_urls: Vec<String> = picker
                .iter()
                .filter(|item| item.kind == "photo")
                .map(|item| item.url.clone())
                .collect();

            if !photo_urls.is_empty() {
                return Ok(Some(DownloadResult::Photos(photo_urls)));
            }

            if let Some(video_item) = picker.iter().find(|item| item.kind == "video") {
                return Ok(Some(DownloadResult::Video(video_item.url.clone())));
            }

            Ok(None)
        }
        _ => {
            if let Some(download_url) = response.get_download_url() {
                Ok(Some(DownloadResult::Video(download_url)))
            } else {
                Ok(None)
            }
        }
    }
}

static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(https?)://[^\s/$.?#].[^\s]*$").unwrap());

pub async fn is_query_url(inline_query: InlineQuery) -> bool {
    URL_REGEX.is_match(&inline_query.query)
}

fn build_results_from_media(
    original_url: &str,
    media: DownloadResult,
) -> Vec<InlineQueryResult> {
    match media {
        DownloadResult::Video(url) => {
            if let (Ok(video_url), Ok(thumb_url)) = (url.parse(), FALLBACK_THUMB_URL.parse()) {
                let mime_type: Mime = "video/mp4".parse().unwrap();
                let video_result = InlineQueryResultVideo::new(
                    original_url,
                    video_url,
                    mime_type,
                    thumb_url,
                    "Video",
                );
                vec![video_result.into()]
            } else {
                vec![]
            }
        }
        DownloadResult::Photos(urls) => urls
            .into_iter()
            .enumerate()
            .filter_map(|(i, url_str)| {
                if let (Ok(photo_url), Ok(thumb_url)) = (url_str.parse(), url_str.parse()) {
                    let photo_result = InlineQueryResultPhoto::new(
                        format!("{}_{}", original_url, i),
                        photo_url,
                        thumb_url,
                    );
                    Some(photo_result.into())
                } else {
                    None
                }
            })
            .collect(),
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

    let user_id_str = q.from.id.to_string();
    let redis = config.get_redis_client();
    let cache_key = format!("cobalt_cache:{}", url);

    let results = if let Ok(Some(cached_result)) = redis.get::<DownloadResult>(&cache_key).await {
        build_results_from_media(url, cached_result)
    } else {
        let settings = Settings::get_or_create(&user_id_str, "user").await?;
        let cobalt_client = config.get_cobalt_client();
        let result = resolve_download_url(url, &settings, cobalt_client).await;

        match result {
            Ok(Some(download_result)) => {
                let ttl_24_hours = 86400;
                let result_to_cache = download_result.clone();
                if let Err(e) = redis.set(&cache_key, &result_to_cache, ttl_24_hours).await {
                    log::error!("Failed to cache cobalt result: {}", e);
                }
                build_results_from_media(url, download_result)
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

    bot.answer_inline_query(q.id, results).await?;

    Ok(())
}