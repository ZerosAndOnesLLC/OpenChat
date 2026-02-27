use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::ApiResult;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub step_order: i32,
    pub action_type: String,
    pub action_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_data: serde_json::Value,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowExecutionStep {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_id: Uuid,
    pub status: String,
    pub input_data: Option<serde_json::Value>,
    pub output_data: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl Workflow {
    pub async fn create(
        pool: &PgPool,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        trigger_config: &serde_json::Value,
        created_by: Uuid,
    ) -> ApiResult<Workflow> {
        let workflow = sqlx::query_as::<_, Workflow>(
            r#"INSERT INTO workflows (id, org_id, name, description, trigger_type, trigger_config, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(trigger_config)
        .bind(created_by)
        .fetch_one(pool)
        .await?;

        Ok(workflow)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Workflow>> {
        let workflow = sqlx::query_as::<_, Workflow>("SELECT * FROM workflows WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(workflow)
    }

    pub async fn list_by_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<Workflow>> {
        let workflows = sqlx::query_as::<_, Workflow>(
            "SELECT * FROM workflows WHERE org_id = $1 ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(pool)
        .await?;

        Ok(workflows)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        trigger_type: Option<&str>,
        trigger_config: Option<&serde_json::Value>,
    ) -> ApiResult<Workflow> {
        let workflow = sqlx::query_as::<_, Workflow>(
            r#"UPDATE workflows
               SET name = COALESCE($2, name),
                   description = COALESCE($3, description),
                   trigger_type = COALESCE($4, trigger_type),
                   trigger_config = COALESCE($5, trigger_config),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(trigger_type)
        .bind(trigger_config)
        .fetch_one(pool)
        .await?;

        Ok(workflow)
    }

    pub async fn set_enabled(pool: &PgPool, id: Uuid, enabled: bool) -> ApiResult<Workflow> {
        let workflow = sqlx::query_as::<_, Workflow>(
            r#"UPDATE workflows SET enabled = $2, updated_at = NOW() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(enabled)
        .fetch_one(pool)
        .await?;

        Ok(workflow)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM workflows WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn find_enabled_by_trigger(
        pool: &PgPool,
        org_id: Uuid,
        trigger_type: &str,
    ) -> ApiResult<Vec<Workflow>> {
        let workflows = sqlx::query_as::<_, Workflow>(
            r#"SELECT * FROM workflows
               WHERE org_id = $1 AND trigger_type = $2 AND enabled = true"#,
        )
        .bind(org_id)
        .bind(trigger_type)
        .fetch_all(pool)
        .await?;

        Ok(workflows)
    }
}

impl WorkflowStep {
    pub async fn create(
        pool: &PgPool,
        workflow_id: Uuid,
        step_order: i32,
        action_type: &str,
        action_config: &serde_json::Value,
    ) -> ApiResult<WorkflowStep> {
        let step = sqlx::query_as::<_, WorkflowStep>(
            r#"INSERT INTO workflow_steps (id, workflow_id, step_order, action_type, action_config)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(workflow_id)
        .bind(step_order)
        .bind(action_type)
        .bind(action_config)
        .fetch_one(pool)
        .await?;

        Ok(step)
    }

    pub async fn list_by_workflow(pool: &PgPool, workflow_id: Uuid) -> ApiResult<Vec<WorkflowStep>> {
        let steps = sqlx::query_as::<_, WorkflowStep>(
            "SELECT * FROM workflow_steps WHERE workflow_id = $1 ORDER BY step_order ASC",
        )
        .bind(workflow_id)
        .fetch_all(pool)
        .await?;

        Ok(steps)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        action_type: Option<&str>,
        action_config: Option<&serde_json::Value>,
    ) -> ApiResult<WorkflowStep> {
        let step = sqlx::query_as::<_, WorkflowStep>(
            r#"UPDATE workflow_steps
               SET action_type = COALESCE($2, action_type),
                   action_config = COALESCE($3, action_config)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(action_type)
        .bind(action_config)
        .fetch_one(pool)
        .await?;

        Ok(step)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM workflow_steps WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete_all_for_workflow(pool: &PgPool, workflow_id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM workflow_steps WHERE workflow_id = $1")
            .bind(workflow_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn reorder(pool: &PgPool, workflow_id: Uuid, step_ids: &[Uuid]) -> ApiResult<()> {
        let mut tx = pool.begin().await?;

        // Temporarily set all to high values to avoid unique constraint violations
        sqlx::query("UPDATE workflow_steps SET step_order = step_order + 10000 WHERE workflow_id = $1")
            .bind(workflow_id)
            .execute(&mut *tx)
            .await?;

        for (i, step_id) in step_ids.iter().enumerate() {
            sqlx::query("UPDATE workflow_steps SET step_order = $1 WHERE id = $2 AND workflow_id = $3")
                .bind(i as i32)
                .bind(step_id)
                .bind(workflow_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

impl WorkflowExecution {
    pub async fn create(
        pool: &PgPool,
        workflow_id: Uuid,
        trigger_data: &serde_json::Value,
    ) -> ApiResult<WorkflowExecution> {
        let execution = sqlx::query_as::<_, WorkflowExecution>(
            r#"INSERT INTO workflow_executions (id, workflow_id, trigger_data)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(workflow_id)
        .bind(trigger_data)
        .fetch_one(pool)
        .await?;

        Ok(execution)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<WorkflowExecution>> {
        let execution =
            sqlx::query_as::<_, WorkflowExecution>("SELECT * FROM workflow_executions WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        Ok(execution)
    }

    pub async fn list_by_workflow(
        pool: &PgPool,
        workflow_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> ApiResult<Vec<WorkflowExecution>> {
        let executions = sqlx::query_as::<_, WorkflowExecution>(
            r#"SELECT * FROM workflow_executions
               WHERE workflow_id = $1
               ORDER BY started_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(workflow_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(executions)
    }

    pub async fn set_completed(pool: &PgPool, id: Uuid) -> ApiResult<WorkflowExecution> {
        let execution = sqlx::query_as::<_, WorkflowExecution>(
            r#"UPDATE workflow_executions
               SET status = 'completed', completed_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(execution)
    }

    pub async fn set_failed(pool: &PgPool, id: Uuid, error_message: &str) -> ApiResult<WorkflowExecution> {
        let execution = sqlx::query_as::<_, WorkflowExecution>(
            r#"UPDATE workflow_executions
               SET status = 'failed', completed_at = NOW(), error_message = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(pool)
        .await?;

        Ok(execution)
    }

    pub async fn set_status(pool: &PgPool, id: Uuid, status: &str) -> ApiResult<WorkflowExecution> {
        let execution = sqlx::query_as::<_, WorkflowExecution>(
            r#"UPDATE workflow_executions
               SET status = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(pool)
        .await?;

        Ok(execution)
    }
}

impl WorkflowExecutionStep {
    pub async fn create_batch(
        pool: &PgPool,
        execution_id: Uuid,
        steps: &[WorkflowStep],
    ) -> ApiResult<Vec<WorkflowExecutionStep>> {
        let mut result = Vec::with_capacity(steps.len());

        for step in steps {
            let exec_step = sqlx::query_as::<_, WorkflowExecutionStep>(
                r#"INSERT INTO workflow_execution_steps (id, execution_id, step_id)
                   VALUES ($1, $2, $3)
                   RETURNING *"#,
            )
            .bind(Uuid::new_v4())
            .bind(execution_id)
            .bind(step.id)
            .fetch_one(pool)
            .await?;

            result.push(exec_step);
        }

        Ok(result)
    }

    pub async fn list_by_execution(
        pool: &PgPool,
        execution_id: Uuid,
    ) -> ApiResult<Vec<WorkflowExecutionStep>> {
        let steps = sqlx::query_as::<_, WorkflowExecutionStep>(
            r#"SELECT wes.* FROM workflow_execution_steps wes
               JOIN workflow_steps ws ON wes.step_id = ws.id
               WHERE wes.execution_id = $1
               ORDER BY ws.step_order ASC"#,
        )
        .bind(execution_id)
        .fetch_all(pool)
        .await?;

        Ok(steps)
    }

    pub async fn set_running(pool: &PgPool, id: Uuid, input_data: &serde_json::Value) -> ApiResult<WorkflowExecutionStep> {
        let step = sqlx::query_as::<_, WorkflowExecutionStep>(
            r#"UPDATE workflow_execution_steps
               SET status = 'running', input_data = $2, started_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(input_data)
        .fetch_one(pool)
        .await?;

        Ok(step)
    }

    pub async fn set_completed(
        pool: &PgPool,
        id: Uuid,
        output_data: &serde_json::Value,
    ) -> ApiResult<WorkflowExecutionStep> {
        let step = sqlx::query_as::<_, WorkflowExecutionStep>(
            r#"UPDATE workflow_execution_steps
               SET status = 'completed', output_data = $2, completed_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(output_data)
        .fetch_one(pool)
        .await?;

        Ok(step)
    }

    pub async fn set_failed(pool: &PgPool, id: Uuid, error_message: &str) -> ApiResult<WorkflowExecutionStep> {
        let step = sqlx::query_as::<_, WorkflowExecutionStep>(
            r#"UPDATE workflow_execution_steps
               SET status = 'failed', completed_at = NOW(), error_message = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(pool)
        .await?;

        Ok(step)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowForm {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub step_id: Uuid,
    pub execution_id: Uuid,
    pub title: String,
    pub fields: serde_json::Value,
    pub target_user_id: Uuid,
    pub submitted_by: Option<Uuid>,
    pub submitted_data: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
}

impl WorkflowForm {
    pub async fn create(
        pool: &PgPool,
        workflow_id: Uuid,
        step_id: Uuid,
        execution_id: Uuid,
        title: &str,
        fields: &serde_json::Value,
        target_user_id: Uuid,
    ) -> ApiResult<WorkflowForm> {
        let form = sqlx::query_as::<_, WorkflowForm>(
            r#"INSERT INTO workflow_forms (id, workflow_id, step_id, execution_id, title, fields, target_user_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(workflow_id)
        .bind(step_id)
        .bind(execution_id)
        .bind(title)
        .bind(fields)
        .bind(target_user_id)
        .fetch_one(pool)
        .await?;

        Ok(form)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<WorkflowForm>> {
        let form = sqlx::query_as::<_, WorkflowForm>("SELECT * FROM workflow_forms WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(form)
    }

    pub async fn list_pending_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<WorkflowForm>> {
        let forms = sqlx::query_as::<_, WorkflowForm>(
            "SELECT * FROM workflow_forms WHERE target_user_id = $1 AND status = 'pending' ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(forms)
    }

    pub async fn submit(
        pool: &PgPool,
        id: Uuid,
        submitted_by: Uuid,
        submitted_data: &serde_json::Value,
    ) -> ApiResult<WorkflowForm> {
        let form = sqlx::query_as::<_, WorkflowForm>(
            r#"UPDATE workflow_forms
               SET status = 'submitted', submitted_by = $2, submitted_data = $3, submitted_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(submitted_by)
        .bind(submitted_data)
        .fetch_one(pool)
        .await?;

        Ok(form)
    }
}
