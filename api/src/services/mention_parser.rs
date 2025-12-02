use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;
use crate::models::mention::{CreateMention, MentionType};
use crate::models::user::User;

/// Parsed mention information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedMention {
    pub mention_type: MentionType,
    pub mentioned_user_id: Option<Uuid>,
    pub raw_text: String,
}

/// Parse mentions from message content
/// Returns a list of ParsedMention objects
pub async fn parse_mentions(content: &str, org_id: Uuid, pool: &PgPool) -> ApiResult<Vec<ParsedMention>> {
    let mut mentions = Vec::new();

    // Regex for @username, @channel, @here, @everyone
    let mention_regex = Regex::new(r"@(\w+)").unwrap();

    for capture in mention_regex.captures_iter(content) {
        let full_match = capture.get(0).unwrap().as_str();
        let username = capture.get(1).unwrap().as_str();

        // Check for special mentions
        if username.eq_ignore_ascii_case("channel") {
            mentions.push(ParsedMention {
                mention_type: MentionType::Channel,
                mentioned_user_id: None,
                raw_text: full_match.to_string(),
            });
        } else if username.eq_ignore_ascii_case("here") {
            mentions.push(ParsedMention {
                mention_type: MentionType::Here,
                mentioned_user_id: None,
                raw_text: full_match.to_string(),
            });
        } else if username.eq_ignore_ascii_case("everyone") {
            mentions.push(ParsedMention {
                mention_type: MentionType::Everyone,
                mentioned_user_id: None,
                raw_text: full_match.to_string(),
            });
        } else {
            // Look up user by display name
            if let Ok(Some(user)) = find_user_by_display_name(pool, org_id, username).await {
                mentions.push(ParsedMention {
                    mention_type: MentionType::User,
                    mentioned_user_id: Some(user.id),
                    raw_text: full_match.to_string(),
                });
            }
        }
    }

    Ok(mentions)
}

/// Find user by display name (case-insensitive)
async fn find_user_by_display_name(pool: &PgPool, org_id: Uuid, display_name: &str) -> ApiResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE org_id = $1 AND LOWER(display_name) = LOWER($2)
        LIMIT 1
        "#,
    )
    .bind(org_id)
    .bind(display_name)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Convert ParsedMentions to CreateMention structs
pub fn to_create_mentions(parsed_mentions: Vec<ParsedMention>, message_id: Uuid) -> Vec<CreateMention> {
    parsed_mentions
        .into_iter()
        .map(|pm| CreateMention {
            message_id,
            mentioned_user_id: pm.mentioned_user_id,
            mention_type: pm.mention_type,
        })
        .collect()
}

/// Get all channel members for @channel, @here, @everyone mentions
pub async fn get_channel_members(pool: &PgPool, channel_id: Uuid) -> ApiResult<Vec<Uuid>> {
    let user_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM channel_members WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await?;

    Ok(user_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mention_regex() {
        let content = "Hey @john, can you review this? @channel FYI @everyone";
        let regex = Regex::new(r"@(\w+)").unwrap();
        let matches: Vec<String> = regex
            .captures_iter(content)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect();

        assert_eq!(matches, vec!["john", "channel", "everyone"]);
    }
}
