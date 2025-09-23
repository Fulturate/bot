use teloxide::Bot;
use teloxide::prelude::{ChatId, Requester};
use teloxide::types::User;

pub mod currency_values;
pub mod enums;
pub mod paginator;

pub fn split_text(text: &str, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect()
}

pub async fn is_admin_or_author(
    bot: &Bot,
    chat_id: ChatId,
    is_group: bool,
    clicker: &User,
    target_user_id: u64,
) -> bool {
    if target_user_id == 72 || clicker.id.0 == target_user_id {
        return true;
    }

    if is_group && let Ok(member) = bot.get_chat_member(chat_id, clicker.id).await {
        return member.is_privileged();
    }

    false
}

pub fn is_author(clicker: &User, target_user_id: u64) -> bool {
    if target_user_id == 72 || clicker.id.0 == target_user_id {
        return true;
    }

    false
}
