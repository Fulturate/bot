use crate::bot::handlers::keyboards::make_photo_pagination_keyboard;
use crate::bot::inlines::cobalter::DownloadResult;
use crate::config::Config;
use crate::errors::MyError;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, InputFile, InputMedia, InputMediaPhoto};
use teloxide::{ApiError, RequestError};

struct PagingData<'a> {
    original_user_id: u64,
    index: usize,
    total: usize,
    url_hash: &'a str,
}

impl<'a> PagingData<'a> {
    fn from_parts(parts: &'a [&'a str]) -> Option<Self> {
        if parts.len() < 5 {
            return None;
        }
        Some(Self {
            original_user_id: parts.get(1)?.parse().ok()?,
            index: parts.get(2)?.parse().ok()?,
            total: parts.get(3)?.parse().ok()?,
            url_hash: parts.get(4)?,
        })
    }
}

pub async fn handle_cobalt_pagination(
    bot: Bot,
    q: CallbackQuery,
    config: Arc<Config>,
) -> Result<(), MyError> {
    let Some(data) = q.data else { return Ok(()) };
    let parts: Vec<&str> = data.split(':').collect();

    if parts.get(1) == Some(&"noop") {
        bot.answer_callback_query(q.id).await?;
        return Ok(());
    }

    let Some(paging_data) = PagingData::from_parts(&parts) else {
        log::warn!("Invalid callback data format: {}", data);
        return Ok(());
    };

    if q.from.id.0 != paging_data.original_user_id {
        bot.answer_callback_query(q.id)
            .text("Вы не можете использовать эти кнопки.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let cache_key = format!("cobalt_cache:{}", paging_data.url_hash);
    let redis = config.get_redis_client();
    let Ok(Some(DownloadResult::Photos { urls, original_url })) = redis.get(&cache_key).await
    else {
        bot.answer_callback_query(q.id)
            .text("Извините, срок хранения этих фото истёк.")
            .show_alert(true)
            .await?;
        return Ok(());
    };

    let Some(photo_url_str) = urls.get(paging_data.index) else {
        log::error!(
            "Pagination index {} is out of bounds for len {}",
            paging_data.index,
            urls.len()
        );
        return Ok(());
    };
    let Ok(url) = photo_url_str.parse() else {
        log::error!("Failed to parse photo URL: {}", photo_url_str);
        return Ok(());
    };

    let media = InputMedia::Photo(InputMediaPhoto::new(InputFile::url(url)));
    let keyboard = make_photo_pagination_keyboard(
        paging_data.url_hash,
        paging_data.index,
        paging_data.total,
        paging_data.original_user_id,
        &original_url,
    );

    let edit_result = if let Some(msg) = q.message {
        bot.edit_message_media(msg.chat().id, msg.id(), media)
            .reply_markup(keyboard)
            .await
            .map(|_| ())
    } else if let Some(inline_id) = q.inline_message_id {
        bot.edit_message_media_inline(inline_id, media)
            .reply_markup(keyboard)
            .await
            .map(|_| ())
    } else {
        return Ok(());
    };

    if let Err(e) = edit_result {
        if !matches!(e, RequestError::Api(ApiError::MessageNotModified)) {
            log::error!("Failed to edit message for pagination: {}", e);
            bot.answer_callback_query(q.id.clone())
                .text("Не удалось обновить фото.")
                .await?;
        }
    }

    bot.answer_callback_query(q.id).await?;

    Ok(())
}
