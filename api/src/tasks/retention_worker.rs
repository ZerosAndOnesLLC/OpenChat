use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::services::audit_logger::AuditLogger;
use crate::storage::StorageFactory;

const BATCH_SIZE: i64 = 100;

#[derive(sqlx::FromRow)]
struct RetentionPolicyRow {
    #[allow(dead_code)]
    id: Uuid,
    policy_type: String,
    retention_days: i32,
}

#[derive(sqlx::FromRow)]
struct MessageIdRow {
    id: Uuid,
}

#[derive(sqlx::FromRow)]
struct AttachmentRow {
    id: Uuid,
    storage_path: String,
}

pub async fn execute(
    pool: &PgPool,
    storage_factory: &StorageFactory,
    org_id: Uuid,
) -> Result<(), String> {
    info!(org_id = %org_id, "Starting retention enforcement");

    // Check for org-wide legal holds — if any, skip entire org
    let org_hold: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 as v FROM legal_holds WHERE org_id = $1 AND channel_id IS NULL AND enabled = true LIMIT 1",
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to check legal holds: {}", e))?;

    if org_hold.is_some() {
        info!(org_id = %org_id, "Org has active org-wide legal hold, skipping retention");
        return Ok(());
    }

    // Get channel IDs under legal hold
    let held_channel_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT channel_id FROM legal_holds WHERE org_id = $1 AND channel_id IS NOT NULL AND enabled = true",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch held channels: {}", e))?;

    let held_ids: Vec<Uuid> = held_channel_ids.into_iter().map(|(id,)| id).collect();

    // Get enabled retention policies for org
    let policies: Vec<RetentionPolicyRow> = sqlx::query_as(
        "SELECT id, policy_type, retention_days FROM retention_policies WHERE org_id = $1 AND enabled = true",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch retention policies: {}", e))?;

    if policies.is_empty() {
        info!(org_id = %org_id, "No enabled retention policies");
        return Ok(());
    }

    let mut total_messages_deleted = 0i64;
    let mut total_files_deleted = 0i64;

    for policy in &policies {
        let cutoff = chrono::Utc::now()
            - chrono::Duration::days(i64::from(policy.retention_days));

        match policy.policy_type.as_str() {
            "messages" => {
                let (msgs, files) = enforce_message_retention(
                    pool,
                    storage_factory,
                    org_id,
                    &held_ids,
                    cutoff,
                )
                .await?;
                total_messages_deleted += msgs;
                total_files_deleted += files;
            }
            "files" => {
                let files = enforce_file_retention(
                    pool,
                    storage_factory,
                    org_id,
                    &held_ids,
                    cutoff,
                )
                .await?;
                total_files_deleted += files;
            }
            other => {
                warn!(org_id = %org_id, policy_type = %other, "Unknown retention policy type");
            }
        }
    }

    // Audit log the enforcement
    if total_messages_deleted > 0 || total_files_deleted > 0 {
        if let Err(e) = AuditLogger::log(
            pool,
            None,
            "retention.enforced",
            "organization",
            Some(org_id),
            json!({
                "messages_deleted": total_messages_deleted,
                "files_deleted": total_files_deleted,
                "org_id": org_id,
            }),
            None,
        )
        .await
        {
            error!(org_id = %org_id, "Failed to create audit log for retention: {}", e);
        }
    }

    info!(
        org_id = %org_id,
        messages_deleted = total_messages_deleted,
        files_deleted = total_files_deleted,
        "Retention enforcement completed"
    );

    Ok(())
}

async fn enforce_message_retention(
    pool: &PgPool,
    storage_factory: &StorageFactory,
    org_id: Uuid,
    held_channel_ids: &[Uuid],
    cutoff: chrono::DateTime<chrono::Utc>,
) -> Result<(i64, i64), String> {
    let mut total_messages: i64 = 0;
    let mut total_files: i64 = 0;

    loop {
        // Get batch of channel messages older than cutoff, excluding held channels
        let channel_messages: Vec<MessageIdRow> = sqlx::query_as(
            r#"
            SELECT m.id FROM messages m
            JOIN channels c ON m.channel_id = c.id
            WHERE c.org_id = $1
              AND m.created_at < $2
              AND m.deleted_at IS NULL
              AND ($3::uuid[] IS NULL OR m.channel_id != ALL($3))
            LIMIT $4
            "#,
        )
        .bind(org_id)
        .bind(cutoff)
        .bind(if held_channel_ids.is_empty() {
            None
        } else {
            Some(held_channel_ids)
        })
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch channel messages for retention: {}", e))?;

        // Get batch of DM messages older than cutoff
        let dm_messages: Vec<MessageIdRow> = sqlx::query_as(
            r#"
            SELECT m.id FROM messages m
            JOIN direct_messages dm ON m.dm_id = dm.id
            WHERE dm.org_id = $1
              AND m.created_at < $2
              AND m.deleted_at IS NULL
            LIMIT $3
            "#,
        )
        .bind(org_id)
        .bind(cutoff)
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch DM messages for retention: {}", e))?;

        let message_ids: Vec<Uuid> = channel_messages
            .iter()
            .chain(dm_messages.iter())
            .map(|m| m.id)
            .collect();

        if message_ids.is_empty() {
            break;
        }

        // Delete attachments for these messages
        let files_deleted = delete_attachments_for_messages(pool, storage_factory, org_id, &message_ids).await?;
        total_files += files_deleted;

        // Soft-delete the messages
        let deleted = sqlx::query(
            "UPDATE messages SET deleted_at = NOW() WHERE id = ANY($1) AND deleted_at IS NULL",
        )
        .bind(&message_ids)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to soft-delete messages: {}", e))?;

        total_messages += deleted.rows_affected() as i64;

        // If we got fewer than batch size, we're done
        if message_ids.len() < BATCH_SIZE as usize {
            break;
        }
    }

    Ok((total_messages, total_files))
}

async fn enforce_file_retention(
    pool: &PgPool,
    storage_factory: &StorageFactory,
    org_id: Uuid,
    held_channel_ids: &[Uuid],
    cutoff: chrono::DateTime<chrono::Utc>,
) -> Result<i64, String> {
    let mut total_files: i64 = 0;

    loop {
        // Get attachments from channel messages older than cutoff
        let attachments: Vec<AttachmentRow> = sqlx::query_as(
            r#"
            SELECT a.id, a.storage_path FROM attachments a
            JOIN messages m ON a.message_id = m.id
            LEFT JOIN channels c ON m.channel_id = c.id
            LEFT JOIN direct_messages dm ON m.dm_id = dm.id
            WHERE (c.org_id = $1 OR dm.org_id = $1)
              AND m.created_at < $2
              AND m.deleted_at IS NULL
              AND ($3::uuid[] IS NULL OR m.channel_id != ALL($3))
            LIMIT $4
            "#,
        )
        .bind(org_id)
        .bind(cutoff)
        .bind(if held_channel_ids.is_empty() {
            None
        } else {
            Some(held_channel_ids)
        })
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to fetch attachments for file retention: {}", e))?;

        if attachments.is_empty() {
            break;
        }

        // Delete files from storage
        let storage = storage_factory
            .get_storage(org_id)
            .await
            .map_err(|e| format!("Failed to get storage: {}", e))?;

        for attachment in &attachments {
            if let Err(e) = storage.delete(&attachment.storage_path).await {
                warn!(
                    attachment_id = %attachment.id,
                    "Failed to delete file from storage: {}",
                    e
                );
            }
        }

        let attachment_ids: Vec<Uuid> = attachments.iter().map(|a| a.id).collect();
        let count = attachment_ids.len() as i64;

        // Delete attachment records
        sqlx::query("DELETE FROM attachments WHERE id = ANY($1)")
            .bind(&attachment_ids)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to delete attachment records: {}", e))?;

        total_files += count;

        if count < BATCH_SIZE {
            break;
        }
    }

    Ok(total_files)
}

async fn delete_attachments_for_messages(
    pool: &PgPool,
    storage_factory: &StorageFactory,
    org_id: Uuid,
    message_ids: &[Uuid],
) -> Result<i64, String> {
    let attachments: Vec<AttachmentRow> = sqlx::query_as(
        "SELECT id, storage_path FROM attachments WHERE message_id = ANY($1)",
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch attachments: {}", e))?;

    if attachments.is_empty() {
        return Ok(0);
    }

    let storage = storage_factory
        .get_storage(org_id)
        .await
        .map_err(|e| format!("Failed to get storage: {}", e))?;

    for attachment in &attachments {
        if let Err(e) = storage.delete(&attachment.storage_path).await {
            warn!(
                attachment_id = %attachment.id,
                "Failed to delete file from storage during retention: {}",
                e
            );
        }
    }

    let ids: Vec<Uuid> = attachments.iter().map(|a| a.id).collect();
    let count = ids.len() as i64;

    sqlx::query("DELETE FROM attachments WHERE id = ANY($1)")
        .bind(&ids)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete attachment records: {}", e))?;

    Ok(count)
}
