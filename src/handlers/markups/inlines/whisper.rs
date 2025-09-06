use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::AnswerInlineQuerySetters;
use teloxide::prelude::{Requester, UserId};
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
    InlineQueryResultArticle, InputMessageContent, InputMessageContentText, ParseMode,
};
use uuid::Uuid;

use crate::config::Config;
use crate::util::errors::MyError;
use log::error;
use teloxide::utils::html;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Recipient {
    pub id: Option<u64>,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Whisper {
    pub sender_id: u64,
    pub sender_first_name: String,
    pub content: String,
    pub recipients: Vec<Recipient>,
}

fn generate_recipient_hash(person: &Recipient) -> String {
    let mut s = DefaultHasher::new();
    person.id.hash(&mut s);
    person.username.hash(&mut s);

    format!("{:x}", s.finish())
}

fn parse_query(query: &str) -> (String, Vec<String>) {
    let mut recipients = Vec::new();
    let mut content_end_index = query.len();

    for part in query.split_whitespace().rev() {
        if part.starts_with('@') && part.len() > 1 {
            recipients.push(part.to_string());
            content_end_index = query.rfind(part).unwrap_or(query.len());
        } else if part.parse::<u64>().is_ok() {
            recipients.push(part.to_string());
            content_end_index = query.rfind(part).unwrap_or(query.len());
        } else {
            break;
        }
    }
    recipients.reverse();

    let content = query[..content_end_index].trim().to_string();
    (content, recipients)
}

async fn update_recents(
    config: &Config,
    user_id: u64,
    new_recipients: &[Recipient],
) -> Result<(), MyError> {
    let redis_key = format!("whisper_recents:{}", user_id);
    let mut recents: Vec<Recipient> = config
        .get_redis_client()
        .get(&redis_key)
        .await?
        .unwrap_or_default();

    for new_recipient in new_recipients.iter().rev() {
        recents.retain(|r| {
            let is_same_id = new_recipient.id.is_some() && r.id == new_recipient.id;
            let is_same_username =
                new_recipient.username.is_some() && r.username == new_recipient.username;
            !is_same_id && !is_same_username
        });
        recents.insert(0, new_recipient.clone());
    }

    recents.truncate(5);

    config
        .get_redis_client()
        .set(&redis_key, &recents, 86400 * 30)
        .await?;
    Ok(())
}

pub async fn handle_whisper_inline(
    bot: Bot,
    q: InlineQuery,
    config: Arc<Config>,
) -> Result<(), MyError> {
    if q.query.is_empty() {
        let article = InlineQueryResultArticle::new(
            "whisper_help",
            "Как использовать шепот?",
            InputMessageContent::Text(InputMessageContentText::new(
                "Начните вводить сообщение, а в конце укажите получателей через @username или их Telegram ID.",
            )),
        )
            .description("Пример: Привет! @username 123456789");

        bot.answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
            .cache_time(5)
            .await?;
        return Ok(());
    }

    let (content, recipient_identifiers) = parse_query(&q.query);
    let sender = q.from.clone();

    if content.is_empty() {
        return Ok(());
    }

    if recipient_identifiers.is_empty() {
        let redis_key = format!("whisper_recents:{}", q.from.id.0);
        let recents: Option<Vec<Recipient>> = config.get_redis_client().get(&redis_key).await?;

        let mut results = Vec::new();
        if let Some(recents) = recents {
            for person in recents {
                let query_filler = if let Some(u) = &person.username {
                    format!("@{}", u)
                } else if let Some(id) = person.id {
                    id.to_string()
                } else {
                    continue;
                };

                let keyboard = InlineKeyboardMarkup::new(vec![vec![
                    InlineKeyboardButton::switch_inline_query_current_chat(
                        format!("Выбрать {}", person.first_name),
                        format!("{} {} ", q.query.trim(), query_filler),
                    ),
                ]]);

                let article = InlineQueryResultArticle::new(
                    format!("recent_{}", generate_recipient_hash(&person)),
                    format!("✍️ Написать {}", person.first_name),
                    InputMessageContent::Text(InputMessageContentText::new(format!(
                        "Нажмите кнопку ниже, чтобы начать шепот для {}",
                        person.first_name
                    ))),
                )
                .description("Нажмите кнопку ниже, чтобы выбрать этого пользователя")
                .reply_markup(keyboard);
                results.push(InlineQueryResult::Article(article));
            }
        }

        let article = InlineQueryResultArticle::new(
            "whisper_no_recipients",
            "Кому шептать?",
            InputMessageContent::Text(InputMessageContentText::new(
                "Укажите получателей, добавив их юзернеймы (@username) или ID в конце сообщения.",
            )),
        )
        .description("Вы не указали получателя.");
        results.push(InlineQueryResult::Article(article));

        bot.answer_inline_query(q.id, results)
            .cache_time(10)
            .await?;
        return Ok(());
    }

    let mut recipients: Vec<Recipient> = Vec::new();
    for identifier in &recipient_identifiers {
        if identifier.starts_with('@') {
            let username = identifier[1..].to_string();
            recipients.push(Recipient {
                id: None,
                first_name: username.clone(),
                username: Some(username.to_lowercase()),
            });
        } else if let Ok(id) = identifier.parse::<u64>() {
            recipients.push(Recipient {
                id: Some(id),
                first_name: format!("{}", id),
                username: None,
            });
        }
    }

    recipients.push(Recipient {
        id: Some(sender.id.0),
        first_name: sender.first_name.clone(),
        username: sender.username.clone(),
    });

    let recipients_for_recents: Vec<Recipient> = recipients
        .iter()
        .filter(|r| r.id != Some(sender.id.0))
        .cloned()
        .collect();

    if !recipients_for_recents.is_empty() {
        if let Err(e) = update_recents(&config, sender.id.0, &recipients_for_recents).await {
            error!("Failed to update recent contacts: {:?}", e);
        }
    }

    let whisper_id = Uuid::new_v4().to_string();
    let whisper = Whisper {
        sender_id: sender.id.0,
        sender_first_name: sender.first_name.clone(),
        content: content.clone(),
        recipients,
    };

    let redis_key = format!("whisper:{}", whisper_id);
    config
        .get_redis_client()
        .set(&redis_key, &whisper, 86400)
        .await?;

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("👁️ Прочитать", format!("whisper_read_{}", whisper_id)),
        InlineKeyboardButton::callback("🗑️ Забыть", format!("whisper_forget_{}", whisper_id)),
    ]]);

    let recipients_str = whisper
        .recipients
        .iter()
        .filter(|r| r.id != Some(sender.id.0))
        .map(|r| {
            if let Some(id) = r.id {
                html::user_mention(UserId(id), &r.first_name)
            } else {
                format!("@{}", html::escape(&r.first_name))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let message_text = format!(
        "🤫 {} шепчет для {}",
        whisper.sender_first_name, recipients_str
    );

    let article = InlineQueryResultArticle::new(
        whisper_id,
        "Нажмите, чтобы отправить шепот",
        InputMessageContent::Text(
            InputMessageContentText::new(message_text).parse_mode(ParseMode::Html),
        ),
    )
    .description(format!("Сообщение: {}", content))
    .reply_markup(keyboard);

    if let Err(e) = bot
        .answer_inline_query(q.id, vec![InlineQueryResult::Article(article)])
        .cache_time(0)
        .await
    {
        error!("Failed to answer whisper inline query: {:?}", e);
    }

    Ok(())
}

pub async fn is_whisper_query(_q: InlineQuery) -> bool {
    true
}
