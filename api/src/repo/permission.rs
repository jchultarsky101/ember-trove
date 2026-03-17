use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{
    EmberTroveError,
    id::{NodeId, PermissionId},
    permission::{GrantPermissionRequest, Permission, PermissionRole},
};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait PermissionRepo: Send + Sync {
    async fn grant(
        &self,
        node_id: NodeId,
        granted_by: &str,
        req: GrantPermissionRequest,
    ) -> Result<Permission, EmberTroveError>;

    async fn revoke(&self, id: PermissionId) -> Result<(), EmberTroveError>;

    async fn list(&self, node_id: NodeId) -> Result<Vec<Permission>, EmberTroveError>;

    async fn find(
        &self,
        node_id: NodeId,
        subject_id: &str,
    ) -> Result<Option<Permission>, EmberTroveError>;
}

pub struct PgPermissionRepo {
    pool: PgPool,
}

impl PgPermissionRepo {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PermissionRow {
    id: Uuid,
    node_id: Uuid,
    subject_id: String,
    role: String,
    granted_by: String,
    created_at: DateTime<Utc>,
}

fn parse_role(s: &str) -> Result<PermissionRole, EmberTroveError> {
    match s {
        "owner" => Ok(PermissionRole::Owner),
        "editor" => Ok(PermissionRole::Editor),
        "viewer" => Ok(PermissionRole::Viewer),
        other => Err(EmberTroveError::Internal(format!(
            "unknown permission_role: {other}"
        ))),
    }
}

fn role_to_str(role: &PermissionRole) -> &'static str {
    match role {
        PermissionRole::Owner => "owner",
        PermissionRole::Editor => "editor",
        PermissionRole::Viewer => "viewer",
    }
}

impl PermissionRow {
    fn into_permission(self) -> Result<Permission, EmberTroveError> {
        Ok(Permission {
            id: PermissionId(self.id),
            node_id: NodeId(self.node_id),
            subject_id: self.subject_id,
            role: parse_role(&self.role)?,
            granted_by: self.granted_by,
            created_at: self.created_at,
        })
    }
}

#[async_trait]
impl PermissionRepo for PgPermissionRepo {
    async fn grant(
        &self,
        node_id: NodeId,
        granted_by: &str,
        req: GrantPermissionRequest,
    ) -> Result<Permission, EmberTroveError> {
        let role_str = role_to_str(&req.role);
        let row = sqlx::query_as::<_, PermissionRow>(
            r#"
            INSERT INTO permissions (node_id, subject_id, role, granted_by)
            VALUES ($1, $2, $3::permission_role, $4)
            ON CONFLICT (node_id, subject_id)
                DO UPDATE SET role = EXCLUDED.role, granted_by = EXCLUDED.granted_by
            RETURNING id, node_id, subject_id, role::text, granted_by, created_at
            "#,
        )
        .bind(node_id.inner())
        .bind(&req.subject_id)
        .bind(role_str)
        .bind(granted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("grant permission failed: {e}")))?;

        row.into_permission()
    }

    async fn revoke(&self, id: PermissionId) -> Result<(), EmberTroveError> {
        let result = sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(id.inner())
            .execute(&self.pool)
            .await
            .map_err(|e| EmberTroveError::Internal(format!("revoke permission failed: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(EmberTroveError::NotFound(format!(
                "permission {id} not found"
            )));
        }
        Ok(())
    }

    async fn list(&self, node_id: NodeId) -> Result<Vec<Permission>, EmberTroveError> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, node_id, subject_id, role::text, granted_by, created_at
            FROM permissions
            WHERE node_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(node_id.inner())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("list permissions failed: {e}")))?;

        rows.into_iter().map(|r| r.into_permission()).collect()
    }

    async fn find(
        &self,
        node_id: NodeId,
        subject_id: &str,
    ) -> Result<Option<Permission>, EmberTroveError> {
        let row = sqlx::query_as::<_, PermissionRow>(
            r#"
            SELECT id, node_id, subject_id, role::text, granted_by, created_at
            FROM permissions
            WHERE node_id = $1 AND subject_id = $2
            "#,
        )
        .bind(node_id.inner())
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EmberTroveError::Internal(format!("find permission failed: {e}")))?;

        row.map(|r| r.into_permission()).transpose()
    }
}
