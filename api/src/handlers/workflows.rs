use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{ApiError, ApiResult},
    models::user::User,
    models::workflow::{Workflow, WorkflowExecution, WorkflowExecutionStep, WorkflowForm, WorkflowStep},
    services::{tv_api::TokenClaims, workflow_engine},
    websocket::server::WsServer,
};

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub steps: Vec<CreateWorkflowStepRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowStepRequest {
    pub action_type: String,
    pub action_config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<serde_json::Value>,
    pub steps: Option<Vec<CreateWorkflowStepRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct ListExecutionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
    pub steps: Vec<WorkflowStepResponse>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowStepResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub step_order: i32,
    pub action_type: String,
    pub action_config: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct WorkflowListItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub enabled: bool,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_data: serde_json::Value,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<ExecutionStepResponse>>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStepResponse {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_id: Uuid,
    pub status: String,
    pub input_data: Option<serde_json::Value>,
    pub output_data: Option<serde_json::Value>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_message: Option<String>,
}

const VALID_TRIGGER_TYPES: &[&str] = &[
    "message_posted",
    "reaction_added",
    "channel_join",
    "scheduled",
    "webhook",
    "slash_command",
];

const VALID_ACTION_TYPES: &[&str] = &[
    "send_message",
    "create_form",
    "call_webhook",
    "add_reaction",
    "create_channel",
    "invite_to_channel",
    "update_channel_topic",
    "delay",
];

fn validate_trigger_type(trigger_type: &str) -> ApiResult<()> {
    if !VALID_TRIGGER_TYPES.contains(&trigger_type) {
        return Err(ApiError::BadRequest(format!(
            "Invalid trigger type: {}. Must be one of: {}",
            trigger_type,
            VALID_TRIGGER_TYPES.join(", ")
        )));
    }
    Ok(())
}

fn validate_action_type(action_type: &str) -> ApiResult<()> {
    if !VALID_ACTION_TYPES.contains(&action_type) {
        return Err(ApiError::BadRequest(format!(
            "Invalid action type: {}. Must be one of: {}",
            action_type,
            VALID_ACTION_TYPES.join(", ")
        )));
    }
    Ok(())
}

impl From<Workflow> for WorkflowListItem {
    fn from(w: Workflow) -> Self {
        Self {
            id: w.id,
            name: w.name,
            description: w.description,
            trigger_type: w.trigger_type,
            enabled: w.enabled,
            created_by: w.created_by,
            created_at: w.created_at.to_rfc3339(),
            updated_at: w.updated_at.to_rfc3339(),
        }
    }
}

impl From<WorkflowStep> for WorkflowStepResponse {
    fn from(s: WorkflowStep) -> Self {
        Self {
            id: s.id,
            workflow_id: s.workflow_id,
            step_order: s.step_order,
            action_type: s.action_type,
            action_config: s.action_config,
            created_at: s.created_at.to_rfc3339(),
        }
    }
}

impl From<WorkflowExecution> for ExecutionResponse {
    fn from(e: WorkflowExecution) -> Self {
        Self {
            id: e.id,
            workflow_id: e.workflow_id,
            trigger_data: e.trigger_data,
            status: e.status,
            started_at: e.started_at.to_rfc3339(),
            completed_at: e.completed_at.map(|dt| dt.to_rfc3339()),
            error_message: e.error_message,
            steps: None,
        }
    }
}

impl From<WorkflowExecutionStep> for ExecutionStepResponse {
    fn from(s: WorkflowExecutionStep) -> Self {
        Self {
            id: s.id,
            execution_id: s.execution_id,
            step_id: s.step_id,
            status: s.status,
            input_data: s.input_data,
            output_data: s.output_data,
            started_at: s.started_at.map(|dt| dt.to_rfc3339()),
            completed_at: s.completed_at.map(|dt| dt.to_rfc3339()),
            error_message: s.error_message,
        }
    }
}

/// GET /api/workflows
pub async fn list_workflows(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflows = Workflow::list_by_org(pool.get_ref(), claims.org_id).await?;
    let response: Vec<WorkflowListItem> = workflows.into_iter().map(WorkflowListItem::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/workflows
pub async fn create_workflow(
    pool: web::Data<PgPool>,
    body: web::Json<CreateWorkflowRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    if body.name.trim().is_empty() || body.name.len() > 255 {
        return Err(ApiError::BadRequest(
            "Workflow name must be 1-255 characters".to_string(),
        ));
    }

    validate_trigger_type(&body.trigger_type)?;

    for step in &body.steps {
        validate_action_type(&step.action_type)?;
    }

    // Create workflow + steps atomically
    let mut tx = pool.begin().await?;

    let workflow = sqlx::query_as::<_, Workflow>(
        r#"INSERT INTO workflows (id, org_id, name, description, trigger_type, trigger_config, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(claims.org_id)
    .bind(&body.name)
    .bind(body.description.as_deref())
    .bind(&body.trigger_type)
    .bind(&body.trigger_config)
    .bind(current_user.id)
    .fetch_one(&mut *tx)
    .await?;

    let mut steps = Vec::new();
    for (i, step_req) in body.steps.iter().enumerate() {
        let step = sqlx::query_as::<_, WorkflowStep>(
            r#"INSERT INTO workflow_steps (id, workflow_id, step_order, action_type, action_config)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(workflow.id)
        .bind(i as i32)
        .bind(&step_req.action_type)
        .bind(&step_req.action_config)
        .fetch_one(&mut *tx)
        .await?;

        steps.push(step);
    }

    tx.commit().await?;

    let response = WorkflowResponse {
        id: workflow.id,
        org_id: workflow.org_id,
        name: workflow.name,
        description: workflow.description,
        trigger_type: workflow.trigger_type,
        trigger_config: workflow.trigger_config,
        enabled: workflow.enabled,
        created_by: workflow.created_by,
        created_at: workflow.created_at.to_rfc3339(),
        updated_at: workflow.updated_at.to_rfc3339(),
        steps: steps.into_iter().map(WorkflowStepResponse::from).collect(),
    };

    Ok(HttpResponse::Created().json(response))
}

/// GET /api/workflows/{id}
pub async fn get_workflow(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    let steps = WorkflowStep::list_by_workflow(pool.get_ref(), workflow.id).await?;

    let response = WorkflowResponse {
        id: workflow.id,
        org_id: workflow.org_id,
        name: workflow.name,
        description: workflow.description,
        trigger_type: workflow.trigger_type,
        trigger_config: workflow.trigger_config,
        enabled: workflow.enabled,
        created_by: workflow.created_by,
        created_at: workflow.created_at.to_rfc3339(),
        updated_at: workflow.updated_at.to_rfc3339(),
        steps: steps.into_iter().map(WorkflowStepResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// PUT /api/workflows/{id}
pub async fn update_workflow(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    body: web::Json<UpdateWorkflowRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    if let Some(ref name) = body.name {
        if name.trim().is_empty() || name.len() > 255 {
            return Err(ApiError::BadRequest(
                "Workflow name must be 1-255 characters".to_string(),
            ));
        }
    }

    if let Some(ref trigger_type) = body.trigger_type {
        validate_trigger_type(trigger_type)?;
    }

    let updated = Workflow::update(
        pool.get_ref(),
        *workflow_id,
        body.name.as_deref(),
        body.description.as_deref(),
        body.trigger_type.as_deref(),
        body.trigger_config.as_ref(),
    )
    .await?;

    // Replace steps if provided
    if let Some(ref new_steps) = body.steps {
        for step in new_steps {
            validate_action_type(&step.action_type)?;
        }

        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM workflow_steps WHERE workflow_id = $1")
            .bind(*workflow_id)
            .execute(&mut *tx)
            .await?;

        for (i, step_req) in new_steps.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO workflow_steps (id, workflow_id, step_order, action_type, action_config)
                   VALUES ($1, $2, $3, $4, $5)"#,
            )
            .bind(Uuid::new_v4())
            .bind(*workflow_id)
            .bind(i as i32)
            .bind(&step_req.action_type)
            .bind(&step_req.action_config)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    }

    let steps = WorkflowStep::list_by_workflow(pool.get_ref(), *workflow_id).await?;

    let response = WorkflowResponse {
        id: updated.id,
        org_id: updated.org_id,
        name: updated.name,
        description: updated.description,
        trigger_type: updated.trigger_type,
        trigger_config: updated.trigger_config,
        enabled: updated.enabled,
        created_by: updated.created_by,
        created_at: updated.created_at.to_rfc3339(),
        updated_at: updated.updated_at.to_rfc3339(),
        steps: steps.into_iter().map(WorkflowStepResponse::from).collect(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// DELETE /api/workflows/{id}
pub async fn delete_workflow(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    Workflow::delete(pool.get_ref(), *workflow_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/workflows/{id}/enable
pub async fn enable_workflow(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    let updated = Workflow::set_enabled(pool.get_ref(), *workflow_id, true).await?;

    Ok(HttpResponse::Ok().json(WorkflowListItem::from(updated)))
}

/// POST /api/workflows/{id}/disable
pub async fn disable_workflow(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    let updated = Workflow::set_enabled(pool.get_ref(), *workflow_id, false).await?;

    Ok(HttpResponse::Ok().json(WorkflowListItem::from(updated)))
}

/// GET /api/workflows/{id}/executions
pub async fn list_executions(
    pool: web::Data<PgPool>,
    workflow_id: web::Path<Uuid>,
    query: web::Query<ListExecutionsQuery>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let executions = WorkflowExecution::list_by_workflow(pool.get_ref(), *workflow_id, limit, offset).await?;
    let response: Vec<ExecutionResponse> = executions.into_iter().map(ExecutionResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/workflows/{id}/executions/{eid}
pub async fn get_execution(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let (workflow_id, execution_id) = path.into_inner();

    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    let execution = WorkflowExecution::get_by_id(pool.get_ref(), execution_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Execution not found".to_string()))?;

    if execution.workflow_id != workflow_id {
        return Err(ApiError::NotFound("Execution not found for this workflow".to_string()));
    }

    let exec_steps = WorkflowExecutionStep::list_by_execution(pool.get_ref(), execution_id).await?;

    let mut response = ExecutionResponse::from(execution);
    response.steps = Some(exec_steps.into_iter().map(ExecutionStepResponse::from).collect());

    Ok(HttpResponse::Ok().json(response))
}

/// POST /api/workflows/{id}/test
pub async fn test_workflow(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    workflow_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if workflow.org_id != claims.org_id {
        return Err(ApiError::Authorization(
            "Workflow does not belong to your organization".to_string(),
        ));
    }

    // Build mock trigger data
    let trigger_data = serde_json::json!({
        "test": true,
        "user_id": current_user.id.to_string(),
        "user_name": current_user.display_name,
        "org_id": claims.org_id.to_string(),
    });

    // Fire trigger directly (synchronously for test)
    workflow_engine::check_triggers(
        pool.get_ref(),
        ws_server.get_ref(),
        claims.org_id,
        &workflow.trigger_type,
        trigger_data,
    )
    .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Test workflow triggered"
    })))
}

/// POST /api/workflows/webhook/{workflow_id} — public endpoint, no auth
pub async fn webhook_trigger(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    workflow_id: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> ApiResult<HttpResponse> {
    let workflow = Workflow::get_by_id(pool.get_ref(), *workflow_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Workflow not found".to_string()))?;

    if !workflow.enabled {
        return Err(ApiError::BadRequest("Workflow is not enabled".to_string()));
    }

    if workflow.trigger_type != "webhook" {
        return Err(ApiError::BadRequest(
            "This workflow does not have a webhook trigger".to_string(),
        ));
    }

    let trigger_data = serde_json::json!({
        "webhook_payload": body.into_inner(),
    });

    workflow_engine::check_triggers(
        pool.get_ref(),
        ws_server.get_ref(),
        workflow.org_id,
        "webhook",
        trigger_data,
    )
    .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Webhook received"
    })))
}

// ---- Form endpoints ----

#[derive(Debug, Deserialize)]
pub struct SubmitFormRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct FormResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub title: String,
    pub fields: serde_json::Value,
    pub status: String,
    pub created_at: String,
    pub submitted_at: Option<String>,
}

impl From<WorkflowForm> for FormResponse {
    fn from(f: WorkflowForm) -> Self {
        Self {
            id: f.id,
            workflow_id: f.workflow_id,
            title: f.title,
            fields: f.fields,
            status: f.status,
            created_at: f.created_at.to_rfc3339(),
            submitted_at: f.submitted_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

/// GET /api/forms/pending
pub async fn list_pending_forms(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let forms = WorkflowForm::list_pending_for_user(pool.get_ref(), current_user.id).await?;
    let response: Vec<FormResponse> = forms.into_iter().map(FormResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

/// GET /api/forms/{id}
pub async fn get_form(
    pool: web::Data<PgPool>,
    form_id: web::Path<Uuid>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let form = WorkflowForm::get_by_id(pool.get_ref(), *form_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Form not found".to_string()))?;

    if form.target_user_id != current_user.id {
        return Err(ApiError::Authorization(
            "You are not the target of this form".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().json(FormResponse::from(form)))
}

/// POST /api/forms/{id}/submit
pub async fn submit_form(
    pool: web::Data<PgPool>,
    ws_server: web::Data<actix::Addr<WsServer>>,
    form_id: web::Path<Uuid>,
    body: web::Json<SubmitFormRequest>,
    req: HttpRequest,
) -> ApiResult<HttpResponse> {
    let claims = req
        .extensions()
        .get::<TokenClaims>()
        .cloned()
        .ok_or_else(|| ApiError::Authentication("Missing authentication".to_string()))?;

    let current_user = User::get_by_tv_user_id(pool.get_ref(), claims.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Current user not found".to_string()))?;

    let form = WorkflowForm::get_by_id(pool.get_ref(), *form_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Form not found".to_string()))?;

    if form.target_user_id != current_user.id {
        return Err(ApiError::Authorization(
            "You are not the target of this form".to_string(),
        ));
    }

    if form.status != "pending" {
        return Err(ApiError::BadRequest("Form has already been submitted".to_string()));
    }

    let submitted = WorkflowForm::submit(pool.get_ref(), *form_id, current_user.id, &body.data).await?;

    // Resume workflow execution with form data
    let execution_id = submitted.execution_id;
    let pool_clone = pool.clone();
    let ws_clone = ws_server.clone();
    let form_data = body.data.clone();
    tokio::spawn(async move {
        workflow_engine::resume_after_form(
            pool_clone.get_ref(),
            ws_clone.get_ref(),
            execution_id,
            form_data,
        )
        .await;
    });

    Ok(HttpResponse::Ok().json(FormResponse::from(submitted)))
}
