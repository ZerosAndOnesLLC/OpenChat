use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::outgoing_webhook::{OutgoingWebhook, WebhookDelivery};
use crate::tasks::job_queue;
use crate::models::job::JobType;

type HmacSha256 = Hmac<Sha256>;

/// Process a single webhook delivery job.
pub async fn execute(pool: &PgPool, payload: &JsonValue) -> Result<(), String> {
    let webhook_id = payload
        .get("webhook_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("Missing or invalid webhook_id in payload")?;

    let delivery_id = payload
        .get("delivery_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("Missing or invalid delivery_id in payload")?;

    let event_type = payload
        .get("event_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing event_type in payload")?;

    let event_payload = payload
        .get("event_payload")
        .ok_or("Missing event_payload in payload")?;

    // Load the outgoing webhook
    let webhook = match OutgoingWebhook::get_by_id(pool, webhook_id)
        .await
        .map_err(|e| format!("Failed to load webhook: {}", e))?
    {
        Some(w) if w.enabled => w,
        _ => {
            info!(webhook_id = %webhook_id, "Webhook disabled or deleted, skipping delivery");
            let _ = WebhookDelivery::update_status(
                pool,
                delivery_id,
                "skipped",
                None,
                None,
                Some("Webhook disabled or deleted"),
                1,
            )
            .await;
            return Ok(());
        }
    };

    // Serialize event payload to JSON bytes
    let body_bytes =
        serde_json::to_vec(event_payload).map_err(|e| format!("Failed to serialize payload: {}", e))?;

    // Compute HMAC-SHA256 signature
    let mut mac = HmacSha256::new_from_slice(webhook.signing_secret.as_bytes())
        .map_err(|e| format!("Invalid signing secret: {}", e))?;
    mac.update(&body_bytes);
    let signature: String = mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    // POST to webhook URL
    let client = reqwest::Client::new();
    let result = client
        .post(&webhook.url)
        .header("Content-Type", "application/json")
        .header("X-OpenChat-Signature", format!("sha256={}", signature))
        .header("X-OpenChat-Event", event_type)
        .header("X-OpenChat-Delivery", delivery_id.to_string())
        .body(body_bytes)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match result {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::new());

            // Truncate response body for storage
            let truncated_body = if body.len() > 4096 {
                format!("{}...(truncated)", &body[..4096])
            } else {
                body
            };

            if (200..300).contains(&status_code) {
                info!(
                    webhook_id = %webhook_id,
                    delivery_id = %delivery_id,
                    status = status_code,
                    "Webhook delivered successfully"
                );
                WebhookDelivery::update_status(
                    pool,
                    delivery_id,
                    "delivered",
                    Some(status_code),
                    Some(&truncated_body),
                    None,
                    1,
                )
                .await
                .map_err(|e| format!("Failed to update delivery status: {}", e))?;
                Ok(())
            } else {
                let err_msg = format!("Webhook returned HTTP {}", status_code);
                warn!(
                    webhook_id = %webhook_id,
                    delivery_id = %delivery_id,
                    status = status_code,
                    "Webhook delivery failed"
                );
                WebhookDelivery::update_status(
                    pool,
                    delivery_id,
                    "failed",
                    Some(status_code),
                    Some(&truncated_body),
                    Some(&err_msg),
                    1,
                )
                .await
                .map_err(|e| format!("Failed to update delivery status: {}", e))?;
                Err(err_msg)
            }
        }
        Err(e) => {
            let err_msg = format!("Request failed: {}", e);
            error!(
                webhook_id = %webhook_id,
                delivery_id = %delivery_id,
                "Webhook delivery request failed: {}",
                e
            );
            WebhookDelivery::update_status(
                pool,
                delivery_id,
                "failed",
                None,
                None,
                Some(&err_msg),
                1,
            )
            .await
            .map_err(|e| format!("Failed to update delivery status: {}", e))?;
            Err(err_msg)
        }
    }
}

/// Emit an event to all matching outgoing webhooks for an org.
/// Creates WebhookDelivery records and enqueues jobs for each.
pub async fn emit_event(
    pool: &PgPool,
    redis: &mut redis::aio::MultiplexedConnection,
    org_id: Uuid,
    event_type: &str,
    event_payload: &JsonValue,
) -> Result<(), String> {
    let webhooks = OutgoingWebhook::find_for_event(pool, org_id, event_type)
        .await
        .map_err(|e| format!("Failed to find webhooks for event: {}", e))?;

    if webhooks.is_empty() {
        return Ok(());
    }

    for webhook in &webhooks {
        // Create delivery record
        let delivery = match WebhookDelivery::create(pool, webhook.id, event_type, event_payload).await {
            Ok(d) => d,
            Err(e) => {
                error!(
                    webhook_id = %webhook.id,
                    event_type = %event_type,
                    "Failed to create webhook delivery record: {}",
                    e
                );
                continue;
            }
        };

        // Enqueue the delivery job
        let job_payload = serde_json::json!({
            "webhook_id": webhook.id.to_string(),
            "delivery_id": delivery.id.to_string(),
            "event_type": event_type,
            "event_payload": event_payload,
        });

        if let Err(e) = job_queue::enqueue_job(
            pool,
            redis,
            Some(org_id),
            JobType::WebhookDelivery,
            job_payload,
            None,
        )
        .await
        {
            error!(
                webhook_id = %webhook.id,
                delivery_id = %delivery.id,
                "Failed to enqueue webhook delivery job: {}",
                e
            );
        }
    }

    Ok(())
}
