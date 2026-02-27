use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobType {
    RetentionEnforcement,
    WebhookDelivery,
    ScheduledMessage,
    Reminder,
    EmailNotification,
    WorkflowExecution,
}

impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobType::RetentionEnforcement => write!(f, "retention_enforcement"),
            JobType::WebhookDelivery => write!(f, "webhook_delivery"),
            JobType::ScheduledMessage => write!(f, "scheduled_message"),
            JobType::Reminder => write!(f, "reminder"),
            JobType::EmailNotification => write!(f, "email_notification"),
            JobType::WorkflowExecution => write!(f, "workflow_execution"),
        }
    }
}

impl std::str::FromStr for JobType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "retention_enforcement" => Ok(JobType::RetentionEnforcement),
            "webhook_delivery" => Ok(JobType::WebhookDelivery),
            "scheduled_message" => Ok(JobType::ScheduledMessage),
            "reminder" => Ok(JobType::Reminder),
            "email_notification" => Ok(JobType::EmailNotification),
            "workflow_execution" => Ok(JobType::WorkflowExecution),
            _ => Err(format!("Unknown job type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retry,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Retry => write!(f, "retry"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "completed" => Ok(JobStatus::Completed),
            "failed" => Ok(JobStatus::Failed),
            "retry" => Ok(JobStatus::Retry),
            _ => Err(format!("Unknown job status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Job {
    pub fn job_type_enum(&self) -> Result<JobType, String> {
        self.job_type.parse()
    }

    pub fn status_enum(&self) -> Result<JobStatus, String> {
        self.status.parse()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub org_id: Option<Uuid>,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub max_attempts: Option<i32>,
    pub scheduled_at: Option<DateTime<Utc>>,
}
