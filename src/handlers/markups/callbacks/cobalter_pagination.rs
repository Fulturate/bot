use crate::config::Config;
use crate::handlers::keyboards::make_photo_pagination_keyboard;
use crate::handlers::markups::inlines::cobalter::DownloadResult;
use crate::util::errors::MyError;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, InputFile, InputMedia, InputMediaPhoto, True};
use teloxide::RequestError;
use url::Url;

pub async fn handle_cobalt_pagination(
    bot: Bot,
    q: CallbackQuery,
    config: Arc<Config>,
) -> Result<(), MyError> {
    let CallbackQuery { id, data, message, inline_message_id, from, .. } = q;
    let mut answer = bot.answer_callback_query(id.clone());

    if let Some(data) = data {
        let parts: Vec<&str> = data.split(':').collect();

        if parts.len() >= 5 && parts[0] == "cobalt_page" {
            if parts[1] == "noop" {
                answer.await?;
                return Ok(());
            }

            let Ok(original_user_id) = parts[1].parse::<u64>() else {
                answer.text("Invalid user ID in callback.").await?;
                return Ok(());
            };

            if from.id.0 != original_user_id {
                answer.text("Вы не можете использовать эти кнопки.").show_alert(true).await?;
                return Ok(());
            }

            let url_hash = parts[2];
            let cache_key = format!("cobalt_cache:{}", url_hash);

            let index_res = parts.get(3).and_then(|s| s.parse::<usize>().ok());
            let total_res = parts.get(4).and_then(|s| s.parse::<usize>().ok());

            if let (Some(index), Some(total)) = (index_res, total_res) {
                let redis = config.get_redis_client();
                match redis.get::<DownloadResult>(&cache_key).await {
                    Ok(Some(DownloadResult::Photos { urls, original_url })) => {
                        if let Some(photo_url_str) = urls.get(index) {
                            if let Ok(url) = Url::parse(photo_url_str) {
                                let media = InputMedia::Photo(InputMediaPhoto::new(InputFile::url(url)));
                                let keyboard = make_photo_pagination_keyboard(&cache_key, index, total, original_user_id, &*original_url);

                                let edit_result = if let Some(msg) = message {
                                    bot.edit_message_media(msg.chat().id, msg.id(), media)
                                        .reply_markup(keyboard)
                                        .await
                                        .map(|_| ())
                                } else if let Some(inline_id) = inline_message_id {
                                    bot.edit_message_media_inline(inline_id, media)
                                        .reply_markup(keyboard)
                                        .await
                                        .map(|_| ())
                                } else {
                                    log::warn!("CallbackQuery has neither message nor inline_message_id");
                                    Ok(())
                                };

                                if let Err(RequestError::Api(err)) = &edit_result {
                                    if !err.to_string().contains("message is not modified") {
                                        log::error!("Failed to edit message for pagination: {}", err);
                                        answer = answer.text("Не удалось обновить фото.");
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        answer = answer
                            .text("Извините, срок хранения этих фото истёк.")
                            .show_alert(true);
                    }
                }
            }
        }
    }
    answer.await?;
    Ok(())
}