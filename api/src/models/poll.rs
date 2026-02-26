use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Poll {
    pub id: Uuid,
    pub message_id: Uuid,
    pub org_id: Uuid,
    pub question: String,
    pub options: serde_json::Value,
    pub poll_type: String,
    pub anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PollVote {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub user_id: Uuid,
    pub option_index: i32,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOptionInfo {
    pub index: i32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOptionResult {
    pub index: i32,
    pub text: String,
    pub votes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResults {
    pub poll_id: Uuid,
    pub question: String,
    pub options: Vec<PollOptionResult>,
    pub total_votes: i64,
    pub poll_type: String,
    pub anonymous: bool,
    pub closed: bool,
    pub user_votes: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct VoteCount {
    option_index: i32,
    count: i64,
}

impl Poll {
    pub async fn create(
        pool: &PgPool,
        message_id: Uuid,
        org_id: Uuid,
        question: &str,
        options: &serde_json::Value,
        poll_type: &str,
        anonymous: bool,
        expires_at: Option<DateTime<Utc>>,
        created_by: Uuid,
    ) -> ApiResult<Poll> {
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            INSERT INTO polls (id, message_id, org_id, question, options, poll_type, anonymous, expires_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(message_id)
        .bind(org_id)
        .bind(question)
        .bind(options)
        .bind(poll_type)
        .bind(anonymous)
        .bind(expires_at)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(poll)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Poll>> {
        let poll = sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(poll)
    }

    pub async fn get_by_message_id(pool: &PgPool, message_id: Uuid) -> ApiResult<Option<Poll>> {
        let poll = sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE message_id = $1")
            .bind(message_id)
            .fetch_optional(pool)
            .await?;

        Ok(poll)
    }

    pub async fn get_by_message_ids(
        pool: &PgPool,
        message_ids: &[Uuid],
    ) -> ApiResult<Vec<Poll>> {
        let polls = sqlx::query_as::<_, Poll>(
            "SELECT * FROM polls WHERE message_id = ANY($1)",
        )
        .bind(message_ids)
        .fetch_all(pool)
        .await?;

        Ok(polls)
    }

    /// Vote on a poll. For single-choice polls, removes existing vote first in a transaction.
    pub async fn vote(
        pool: &PgPool,
        poll_id: Uuid,
        user_id: Uuid,
        option_index: i32,
    ) -> ApiResult<()> {
        let poll = Self::get_by_id(pool, poll_id)
            .await?
            .ok_or_else(|| crate::errors::ApiError::NotFound("Poll not found".to_string()))?;

        if poll.closed_at.is_some() {
            return Err(crate::errors::ApiError::BadRequest(
                "Poll is closed".to_string(),
            ));
        }

        if let Some(expires_at) = poll.expires_at {
            if Utc::now() > expires_at {
                return Err(crate::errors::ApiError::BadRequest(
                    "Poll has expired".to_string(),
                ));
            }
        }

        // Validate option_index
        let options: Vec<PollOptionInfo> = serde_json::from_value(poll.options)
            .unwrap_or_default();
        if option_index < 0 || option_index >= options.len() as i32 {
            return Err(crate::errors::ApiError::BadRequest(
                "Invalid option index".to_string(),
            ));
        }

        if poll.poll_type == "single" {
            // Single choice: delete existing votes, then insert
            let mut tx = pool.begin().await?;

            sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2")
                .bind(poll_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                "INSERT INTO poll_votes (id, poll_id, user_id, option_index) VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(poll_id)
            .bind(user_id)
            .bind(option_index)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        } else {
            // Multiple choice: toggle (insert or delete)
            let existing = sqlx::query_as::<_, PollVote>(
                "SELECT * FROM poll_votes WHERE poll_id = $1 AND user_id = $2 AND option_index = $3",
            )
            .bind(poll_id)
            .bind(user_id)
            .bind(option_index)
            .fetch_optional(pool)
            .await?;

            if existing.is_some() {
                sqlx::query(
                    "DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2 AND option_index = $3",
                )
                .bind(poll_id)
                .bind(user_id)
                .bind(option_index)
                .execute(pool)
                .await?;
            } else {
                sqlx::query(
                    "INSERT INTO poll_votes (id, poll_id, user_id, option_index) VALUES ($1, $2, $3, $4)",
                )
                .bind(Uuid::new_v4())
                .bind(poll_id)
                .bind(user_id)
                .bind(option_index)
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn remove_vote(pool: &PgPool, poll_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2")
            .bind(poll_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_results(
        pool: &PgPool,
        poll_id: Uuid,
        current_user_id: Uuid,
    ) -> ApiResult<PollResults> {
        let poll = Self::get_by_id(pool, poll_id)
            .await?
            .ok_or_else(|| crate::errors::ApiError::NotFound("Poll not found".to_string()))?;

        let options: Vec<PollOptionInfo> =
            serde_json::from_value(poll.options).unwrap_or_default();

        // Get vote counts per option
        let vote_counts = sqlx::query_as::<_, VoteCount>(
            r#"
            SELECT option_index, COUNT(*) as count
            FROM poll_votes
            WHERE poll_id = $1
            GROUP BY option_index
            "#,
        )
        .bind(poll_id)
        .fetch_all(pool)
        .await?;

        let count_map: std::collections::HashMap<i32, i64> = vote_counts
            .into_iter()
            .map(|vc| (vc.option_index, vc.count))
            .collect();

        let total_votes: i64 = count_map.values().sum();

        let option_results: Vec<PollOptionResult> = options
            .into_iter()
            .map(|opt| PollOptionResult {
                index: opt.index,
                text: opt.text,
                votes: *count_map.get(&opt.index).unwrap_or(&0),
            })
            .collect();

        // Get current user's votes
        let user_votes = sqlx::query_scalar::<_, i32>(
            "SELECT option_index FROM poll_votes WHERE poll_id = $1 AND user_id = $2",
        )
        .bind(poll_id)
        .bind(current_user_id)
        .fetch_all(pool)
        .await?;

        Ok(PollResults {
            poll_id: poll.id,
            question: poll.question,
            options: option_results,
            total_votes,
            poll_type: poll.poll_type,
            anonymous: poll.anonymous,
            closed: poll.closed_at.is_some(),
            user_votes,
        })
    }

    pub async fn close(pool: &PgPool, id: Uuid) -> ApiResult<Poll> {
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            UPDATE polls SET closed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(poll)
    }
}
