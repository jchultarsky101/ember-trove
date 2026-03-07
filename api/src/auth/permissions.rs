/// Per-node permission check helpers.
///
/// Phase 1 stub — all checks pass through.
/// Phase 7 wires these against the `permissions` table.
use common::auth::AuthClaims;
use common::permission::PermissionRole;

use crate::error::ApiError;

/// Assert that `claims.sub` holds at least `required_role` on `node_id`.
///
/// Phase 1 always returns `Ok(())`.
pub fn require_role(
    _claims: &AuthClaims,
    _node_id: uuid::Uuid,
    _required_role: PermissionRole,
) -> Result<(), ApiError> {
    Ok(())
}

pub fn require_owner(claims: &AuthClaims, node_id: uuid::Uuid) -> Result<(), ApiError> {
    require_role(claims, node_id, PermissionRole::Owner)
}

pub fn require_editor(claims: &AuthClaims, node_id: uuid::Uuid) -> Result<(), ApiError> {
    require_role(claims, node_id, PermissionRole::Editor)
}

pub fn require_viewer(claims: &AuthClaims, node_id: uuid::Uuid) -> Result<(), ApiError> {
    require_role(claims, node_id, PermissionRole::Viewer)
}
