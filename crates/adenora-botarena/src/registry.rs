use adenora_common::error::AdenoraError;
use adenora_common::types::*;
use super::{Bot, BotStats, BotStatus};
use chrono::Utc;
use uuid::Uuid;

pub fn register_bot(
    owner_id: UserId,
    name: String,
    description: Option<String>,
    is_open_source: bool,
    source_url: Option<String>,
    max_name_length: usize,
) -> Result<Bot, AdenoraError> {
    if name.is_empty() || name.len() > max_name_length {
        return Err(AdenoraError::Internal(format!(
            "bot name must be 1-{max_name_length} characters"
        )));
    }

    // Sanitize name — alphanumeric, hyphens, underscores only
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AdenoraError::Internal(
            "bot name must be alphanumeric (hyphens and underscores allowed)".into(),
        ));
    }

    Ok(Bot {
        id: Uuid::new_v4(),
        owner_id,
        name,
        description,
        avatar_url: None,
        is_open_source,
        source_url,
        stats: BotStats::default(),
        status: BotStatus::Active,
        created_at: Utc::now(),
    })
}
