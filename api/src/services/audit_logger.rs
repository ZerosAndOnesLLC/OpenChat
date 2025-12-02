use actix_web::HttpRequest;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::ApiResult,
    models::audit_log::{AuditLog, CreateAuditLog},
};

/// Audit logger service for tracking important actions
pub struct AuditLogger;

impl AuditLogger {
    /// Log an audit event
    pub async fn log(
        pool: &PgPool,
        user_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        metadata: serde_json::Value,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        let (ip_address, user_agent) = if let Some(request) = req {
            (
                Self::extract_ip_address(request),
                Self::extract_user_agent(request),
            )
        } else {
            (None, None)
        };

        let create_log = CreateAuditLog {
            user_id,
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id,
            metadata: Some(metadata),
            ip_address,
            user_agent,
        };

        AuditLog::create(pool, create_log).await
    }

    /// Log a message deletion event
    pub async fn log_message_deleted(
        pool: &PgPool,
        user_id: Uuid,
        message_id: Uuid,
        message_content: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "message.deleted",
            "message",
            Some(message_id),
            json!({
                "content": message_content,
                "deleted_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a channel creation event
    pub async fn log_channel_created(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        channel_name: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "channel.created",
            "channel",
            Some(channel_id),
            json!({
                "name": channel_name,
                "created_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a channel deletion event
    pub async fn log_channel_deleted(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        channel_name: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "channel.deleted",
            "channel",
            Some(channel_id),
            json!({
                "name": channel_name,
                "deleted_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a channel member added event
    pub async fn log_channel_member_added(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        added_user_id: Uuid,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "channel.member_added",
            "channel",
            Some(channel_id),
            json!({
                "added_user_id": added_user_id,
                "added_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a channel member removed event
    pub async fn log_channel_member_removed(
        pool: &PgPool,
        user_id: Uuid,
        channel_id: Uuid,
        removed_user_id: Uuid,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "channel.member_removed",
            "channel",
            Some(channel_id),
            json!({
                "removed_user_id": removed_user_id,
                "removed_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a permission change event
    #[allow(dead_code)]
    pub async fn log_permission_changed(
        pool: &PgPool,
        user_id: Uuid,
        action: &str,
        role_id: Uuid,
        permission_name: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            action,
            "permission",
            Some(role_id),
            json!({
                "permission": permission_name,
                "changed_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a role assignment event
    #[allow(dead_code)]
    pub async fn log_role_assigned(
        pool: &PgPool,
        admin_user_id: Uuid,
        target_user_id: Uuid,
        role_id: Uuid,
        role_name: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(admin_user_id),
            "role.assigned",
            "role",
            Some(role_id),
            json!({
                "target_user_id": target_user_id,
                "role_name": role_name,
                "assigned_by": admin_user_id,
            }),
            req,
        )
        .await
    }

    /// Log a role unassignment event
    #[allow(dead_code)]
    pub async fn log_role_unassigned(
        pool: &PgPool,
        admin_user_id: Uuid,
        target_user_id: Uuid,
        role_id: Uuid,
        role_name: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(admin_user_id),
            "role.unassigned",
            "role",
            Some(role_id),
            json!({
                "target_user_id": target_user_id,
                "role_name": role_name,
                "unassigned_by": admin_user_id,
            }),
            req,
        )
        .await
    }

    /// Log a settings change event
    pub async fn log_settings_updated(
        pool: &PgPool,
        user_id: Uuid,
        setting_type: &str,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "settings.updated",
            "settings",
            None,
            json!({
                "setting_type": setting_type,
                "old_value": old_value,
                "new_value": new_value,
                "updated_by": user_id,
            }),
            req,
        )
        .await
    }

    /// Log a user login event
    #[allow(dead_code)]
    pub async fn log_user_login(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "user.login",
            "user",
            Some(user_id),
            json!({
                "email": email,
            }),
            req,
        )
        .await
    }

    /// Log a user logout event
    #[allow(dead_code)]
    pub async fn log_user_logout(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            Some(user_id),
            "user.logout",
            "user",
            Some(user_id),
            json!({
                "email": email,
            }),
            req,
        )
        .await
    }

    /// Log a failed login attempt
    #[allow(dead_code)]
    pub async fn log_login_failed(
        pool: &PgPool,
        email: &str,
        reason: &str,
        req: Option<&HttpRequest>,
    ) -> ApiResult<AuditLog> {
        Self::log(
            pool,
            None,
            "user.login_failed",
            "user",
            None,
            json!({
                "email": email,
                "reason": reason,
            }),
            req,
        )
        .await
    }

    /// Extract IP address from request, preferring X-Forwarded-For header
    fn extract_ip_address(req: &HttpRequest) -> Option<String> {
        // Check X-Forwarded-For header first (for ALB/CloudFront)
        if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                // Take the first IP in the list (client IP)
                if let Some(client_ip) = forwarded_str.split(',').next() {
                    return Some(client_ip.trim().to_string());
                }
            }
        }

        // Fall back to connection info
        req.connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string())
    }

    /// Extract user agent from request
    fn extract_user_agent(req: &HttpRequest) -> Option<String> {
        req.headers()
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}

/// Helper macro to simplify audit logging in handlers
/// Usage: audit_log!(pool, user_id, "action", "resource_type", resource_id, metadata, req)
#[macro_export]
macro_rules! audit_log {
    ($pool:expr, $user_id:expr, $action:expr, $resource_type:expr, $resource_id:expr, $metadata:expr, $req:expr) => {
        $crate::services::audit_logger::AuditLogger::log(
            $pool,
            $user_id,
            $action,
            $resource_type,
            $resource_id,
            $metadata,
            Some($req),
        )
    };
}
