use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::models::workflow::{Workflow, WorkflowExecution};

/// Execute a workflow job from the job queue.
/// Handles scheduled triggers and resume-after-delay.
pub async fn execute(pool: &PgPool, _redis: &mut redis::aio::MultiplexedConnection, payload: &JsonValue) -> Result<(), String> {
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or("Missing action in workflow job payload")?;

    match action {
        "scheduled_trigger" => {
            let workflow_id = payload
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or("Missing or invalid workflow_id")?;

            let workflow = Workflow::get_by_id(pool, workflow_id)
                .await
                .map_err(|e| format!("Failed to load workflow: {}", e))?
                .ok_or("Workflow not found")?;

            if !workflow.enabled {
                info!(workflow_id = %workflow_id, "Scheduled workflow is disabled, skipping");
                return Ok(());
            }

            info!(workflow_id = %workflow_id, "Executing scheduled workflow trigger");

            let trigger_data = serde_json::json!({
                "scheduled": true,
                "org_id": workflow.org_id.to_string(),
            });

            // We can't easily call check_triggers here without ws_server,
            // so we create the execution directly
            let steps = crate::models::workflow::WorkflowStep::list_by_workflow(pool, workflow_id)
                .await
                .map_err(|e| format!("Failed to load steps: {}", e))?;

            if steps.is_empty() {
                return Ok(());
            }

            let execution = WorkflowExecution::create(pool, workflow_id, &trigger_data)
                .await
                .map_err(|e| format!("Failed to create execution: {}", e))?;

            let exec_steps = crate::models::workflow::WorkflowExecutionStep::create_batch(pool, execution.id, &steps)
                .await
                .map_err(|e| format!("Failed to create execution steps: {}", e))?;

            // Execute steps sequentially (without WS broadcasting since we don't have ws_server)
            let mut context = serde_json::json!({
                "trigger": trigger_data,
                "steps": {},
            });

            for (i, (step, exec_step)) in steps.iter().zip(exec_steps.iter()).enumerate() {
                let input = serde_json::json!({
                    "action_type": step.action_type,
                    "action_config": step.action_config,
                });

                let _ = crate::models::workflow::WorkflowExecutionStep::set_running(pool, exec_step.id, &input).await;

                // For job-queue based execution, we only handle delay and call_webhook
                match step.action_type.as_str() {
                    "call_webhook" => {
                        match execute_webhook_action(&step.action_config).await {
                            Ok(output) => {
                                let _ = crate::models::workflow::WorkflowExecutionStep::set_completed(pool, exec_step.id, &output).await;
                                context["steps"][i.to_string()] = output;
                            }
                            Err(err) => {
                                let _ = crate::models::workflow::WorkflowExecutionStep::set_failed(pool, exec_step.id, &err).await;
                                let _ = WorkflowExecution::set_failed(pool, execution.id, &err).await;
                                return Err(err);
                            }
                        }
                    }
                    "delay" => {
                        let seconds = step.action_config
                            .get("seconds")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                            .min(300);
                        if seconds > 0 {
                            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
                        }
                        let output = serde_json::json!({"delayed_seconds": seconds});
                        let _ = crate::models::workflow::WorkflowExecutionStep::set_completed(pool, exec_step.id, &output).await;
                        context["steps"][i.to_string()] = output;
                    }
                    other => {
                        // Skip actions that require ws_server in job queue context
                        let output = serde_json::json!({"skipped": true, "reason": format!("{} not supported in background job", other)});
                        let _ = crate::models::workflow::WorkflowExecutionStep::set_completed(pool, exec_step.id, &output).await;
                        context["steps"][i.to_string()] = output;
                    }
                }
            }

            let _ = WorkflowExecution::set_completed(pool, execution.id).await;
            info!(execution_id = %execution.id, "Scheduled workflow execution completed");

            Ok(())
        }
        _ => Err(format!("Unknown workflow job action: {}", action)),
    }
}

async fn execute_webhook_action(config: &JsonValue) -> Result<JsonValue, String> {
    let url = config
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("call_webhook: missing url")?;

    let method = config
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("POST");

    let body = config.get("body").cloned().unwrap_or(JsonValue::Null);

    let client = reqwest::Client::new();
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.post(url),
    };

    if method.to_uppercase() != "GET" && !body.is_null() {
        request = request.json(&body);
    }

    let response = request
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Webhook request failed: {}", e))?;

    let status = response.status().as_u16();
    let response_body = response.text().await.unwrap_or_default();

    let truncated = if response_body.len() > 4096 {
        format!("{}...(truncated)", &response_body[..4096])
    } else {
        response_body
    };

    if !(200..300).contains(&status) {
        return Err(format!("Webhook returned HTTP {}: {}", status, truncated));
    }

    Ok(serde_json::json!({
        "status": status,
        "body": truncated,
    }))
}
