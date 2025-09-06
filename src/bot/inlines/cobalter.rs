use crate::bot::handlers::keyboards::{make_photo_pagination_keyboard, make_single_url_keyboard};
use crate::config::Config;
use crate::core::db::schemas::SettingsRepo;
use crate::core::db::schemas::settings::Settings;
use crate::errors::MyError;
use ccobalt::model::request::{DownloadRequest, FilenameStyle, VideoQuality};
use ccobalt::model::response::DownloadResponse;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::types::{
    InlineQuery, InlineQueryResult,
    InlineQueryResultArticle, InlineQueryResultPhoto, InlineQueryResultVideo, InputMessageContent, InputMessageContentText,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownloadResult {
    Video(String),
    Photos {
        urls: Vec<String>,
        original_url: String,
    },
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
                return Ok(Some(DownloadResult::Photos {
                    urls: photo_urls,
                    original_url: url.to_string(),
                }));
            }
            if let Some(video_item) = picker.iter().find(|item| item.kind == "video") {
                return Ok(Some(DownloadResult::Video(video_item.url.clone())));
            }
            Ok(None)
        }
        DownloadResponse::Tunnel { url, filename }
        | DownloadResponse::Redirect { url, filename } => {
            const PHOTO_EXTENSIONS: &[&str] = &[".jpg", ".jpeg", ".png", ".gif", ".webp"];
            let is_photo = PHOTO_EXTENSIONS
                .iter()
                .any(|ext| filename.to_lowercase().ends_with(ext));

            if is_photo {
                Ok(Some(DownloadResult::Photos {
                    urls: vec![url.clone()],
                    original_url: url,
                }))
            } else {
                Ok(Some(DownloadResult::Video(url)))
            }
        }
        _ => Ok(response.get_download_url().map(DownloadResult::Video)),
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

    let url_hash_digest = md5::compute(url);
    let url_hash = format!("{:x}", url_hash_digest);
    let cache_key = format!("cobalt_cache:{}", url_hash);

    let redis = config.get_redis_client();

    let results = if let Ok(Some(cached_result)) = redis.get::<DownloadResult>(&cache_key).await {
        build_results_from_media(url, cached_result, &url_hash, user_id)
    } else {
        let settings = Settings::get_or_create(&user_id_str, "user").await?;
        let cobalt_client = config.get_cobalt_client();
        let result = resolve_download_url(url, &settings, cobalt_client).await;
        match result {
            Ok(Some(download_result)) => {
                let ttl_42_hours = 151_200;
                if let Err(e) = redis.set(&cache_key, &download_result, ttl_42_hours).await {
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

// completely useless
// pub async fn handle_chosen_inline_video(
//     bot: Bot,
//     chosen: ChosenInlineResult,
//     config: Arc<Config>,
// ) -> Result<(), MyError> {
//     if let Some(inline_message_id) = chosen.inline_message_id {
//         if let Some(url_hash) = chosen.result_id.strip_prefix("cobalt_video:") {
//             let redis = config.get_redis_client();
//             let cache_key = format!("cobalt_cache:{}", url_hash);
//
//             if let Ok(Some(DownloadResult::Video(video_url))) =
//                 redis.get::<DownloadResult>(&cache_key).await
//             {
//                 let media =
//                     InputMedia::Video(InputMediaVideo::new(InputFile::url(video_url.parse()?)));
//                 if let Err(e) = bot
//                     .edit_message_media_inline(&inline_message_id, media)
//                     .await
//                 {
//                     log::error!("Failed to edit message with video: {}", e);
//                     bot.edit_message_text_inline(
//                         inline_message_id,
//                         "Ошибка: не удалось отправить видео.",
//                     )
//                     .await?;
//                 }
//             } else {
//                 bot.edit_message_text_inline(
//                     inline_message_id,
//                     "Ошибка: видео не найдено в кэше или срок его хранения истёк.",
//                 )
//                 .await?;
//             }
//         }
//     }
//     Ok(())
// }
