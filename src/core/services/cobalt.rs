use crate::{bot::modules::cobalt::CobaltSettings, errors::MyError};
use ccobalt::model::{
    request::{DownloadRequest, FilenameStyle},
    response::DownloadResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum VideoQuality {
    Q720,
    Q1080,
    Q1440,
    Max,
}

impl VideoQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoQuality::Q720 => "720",
            VideoQuality::Q1080 => "1080",
            VideoQuality::Q1440 => "1440",
            VideoQuality::Max => "max",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "1080" => VideoQuality::Q1080,
            "1440" => VideoQuality::Q1440,
            "max" => VideoQuality::Max,
            _ => VideoQuality::Q720,
        }
    }
}

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
    settings: &CobaltSettings,
    client: &ccobalt::Client,
) -> Result<Option<DownloadResult>, MyError> {
    let cobalt_req = DownloadRequest {
        url: url.to_string(),
        filename_style: Some(FilenameStyle::Pretty),
        video_quality: Some(match settings.video_quality {
            VideoQuality::Q720 => ccobalt::model::request::VideoQuality::Q720,
            VideoQuality::Q1080 => ccobalt::model::request::VideoQuality::Q1080,
            VideoQuality::Q1440 => ccobalt::model::request::VideoQuality::Q1440,
            VideoQuality::Max => ccobalt::model::request::VideoQuality::Max,
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
