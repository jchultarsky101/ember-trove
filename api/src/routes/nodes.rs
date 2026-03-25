use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use bytes::Bytes;
use common::{
    attachment::Attachment,
    auth::AuthClaims,
    edge::EdgeWithTitles,
    id::{NodeId, PermissionId, TagId},
    node::{CreateNodeRequest, Node, NodeListParams, NodeListResponse, NodeTitleEntry, UpdateNodeRequest},
    permission::{GrantPermissionRequest, InviteRequest, Permission, PermissionRole},
    tag::Tag,
};
use garde::Validate;
use uuid::Uuid;

use crate::{
    auth::permissions::{require_editor, require_owner, require_viewer},
    error::ApiError,
    state::AppState,
    wikilink::parse_wikilink_titles,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_nodes).post(create_node))
        .route("/titles", get(list_titles))
        .route("/slug/{slug}", get(get_node_by_slug))
        .route("/{id}", get(get_node).put(update_node).delete(delete_node))
        .route("/{id}/neighbors", get(neighbors))
        .route("/{id}/backlinks", get(backlinks))
        .route("/{id}/edges", get(list_edges_for_node))
        .route("/{id}/tags", get(list_tags_for_node))
        .route("/{id}/tags/{tag_id}", post(attach_tag).delete(detach_tag))
        .route(
            "/{id}/attachments",
            get(list_attachments).post(upload_attachment)
                .layer(DefaultBodyLimit::max(50 * 1024 * 1024)), // 50 MiB upload cap
        )
        .route(
            "/{id}/permissions",
            get(list_permissions).post(grant_permission),
        )
        .route("/{id}/permissions/{perm_id}", delete(revoke_permission))
        .route("/{id}/invite", post(invite))
}

// ── Node list ────────────────────────────────────────────────────────────────

async fn list_nodes(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Query(mut params): Query<NodeListParams>,
) -> Result<Json<NodeListResponse>, ApiError> {
    // Enforce private-by-default: only show nodes the caller owns or has a
    // permission row for.  The client-supplied owner_id filter is ignored.
    params.owner_id = None;
    params.subject_id = Some(claims.sub.clone());

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).min(200);
    let (nodes, total) = state.nodes.list(params).await?;
    let has_more = ((page as u64) * (per_page as u64)) + (nodes.len() as u64) < total;

    Ok(Json(NodeListResponse {
        nodes,
        total,
        page,
        per_page,
        has_more,
    }))
}

// ── Title list (used for wiki-link autocomplete) ─────────────────────────────

async fn list_titles(
    State(state): State<AppState>,
) -> Result<Json<Vec<NodeTitleEntry>>, ApiError> {
    let titles = state.nodes.list_titles().await?;
    Ok(Json(titles))
}

// ── Node CRUD ────────────────────────────────────────────────────────────────

async fn create_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<Node>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let node = state.nodes.create(&claims.sub, req).await?;

    // Auto-grant Owner permission for the creator so that require_role()
    // checks work immediately from the first request.
    let owner_req = GrantPermissionRequest {
        subject_id: claims.sub.clone(),
        role: PermissionRole::Owner,
    };
    state
        .permissions
        .grant(node.id, &claims.sub, owner_req)
        .await
        .map_err(|e| ApiError::Internal(format!("auto-grant owner permission failed: {e}")))?;

    sync_wikilinks(&state, node.id, node.body.as_deref().unwrap_or("")).await?;
    Ok((StatusCode::CREATED, Json(node)))
}

async fn get_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Node>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let node = state.nodes.get(NodeId(id)).await?;
    Ok(Json(node))
}

async fn get_node_by_slug(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(slug): Path<String>,
) -> Result<Json<Node>, ApiError> {
    // Resolve slug → id first so we can check permissions.
    let node = state.nodes.get_by_slug(&slug).await?;
    require_viewer(state.permissions.as_ref(), &claims, node.id).await?;
    Ok(Json(node))
}

async fn update_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<Node>, ApiError> {
    require_editor(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let node = state.nodes.update(NodeId(id), req).await?;
    sync_wikilinks(&state, node.id, node.body.as_deref().unwrap_or("")).await?;
    Ok(Json(node))
}

async fn delete_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_owner(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    state.nodes.delete(NodeId(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn neighbors(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Node>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let nodes = state.nodes.neighbors(NodeId(id)).await?;
    Ok(Json(nodes))
}

async fn backlinks(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Node>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let nodes = state.nodes.backlinks(NodeId(id)).await?;
    Ok(Json(nodes))
}

// ── Phase 4: Edges & Tags ────────────────────────────────────────────────────

async fn list_edges_for_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<EdgeWithTitles>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let edges = state.edges.list_for_node_with_titles(NodeId(id)).await?;
    Ok(Json(edges))
}

async fn list_tags_for_node(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let tags = state.tags.list_for_node(NodeId(id)).await?;
    Ok(Json(tags))
}

async fn attach_tag(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path((id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_editor(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    state.tags.attach(NodeId(id), TagId(tag_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn detach_tag(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path((id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_editor(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    state.tags.detach(NodeId(id), TagId(tag_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Phase 6: Attachments ─────────────────────────────────────────────────────

async fn list_attachments(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Attachment>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let attachments = state.attachments.list(NodeId(id)).await?;
    Ok(Json(attachments))
}

async fn upload_attachment(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Attachment>), ApiError> {
    require_editor(state.permissions.as_ref(), &claims, NodeId(id)).await?;

    // Read the first field — expected to be the file.
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::Validation(format!("multipart error: {e}")))?
        .ok_or_else(|| ApiError::Validation("no file field in request".to_string()))?;

    let filename = sanitize_filename(field.file_name().unwrap_or("upload"));
    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    let data: Bytes = field
        .bytes()
        .await
        .map_err(|e| ApiError::Validation(format!("read multipart field: {e}")))?;
    let size_bytes = data.len() as i64;

    // Generate a unique S3 key: <node_id>/<uuid>/<filename>
    let s3_key = format!("{}/{}/{}", id, Uuid::new_v4(), filename);

    state
        .object_store
        .put(&s3_key, data, &content_type)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

    let attachment = state
        .attachments
        .create(NodeId(id), &filename, &content_type, size_bytes, &s3_key)
        .await?;

    Ok((StatusCode::CREATED, Json(attachment)))
}

// ── Phase 7: Permissions ─────────────────────────────────────────────────────

async fn list_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Permission>>, ApiError> {
    require_viewer(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    let perms = state.permissions.list(NodeId(id)).await?;
    Ok(Json(perms))
}

async fn grant_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<GrantPermissionRequest>,
) -> Result<(StatusCode, Json<Permission>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    require_owner(state.permissions.as_ref(), &claims, NodeId(id)).await?;

    let perm = state
        .permissions
        .grant(NodeId(id), &claims.sub, req)
        .await?;
    Ok((StatusCode::CREATED, Json(perm)))
}

async fn revoke_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path((id, perm_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_owner(state.permissions.as_ref(), &claims, NodeId(id)).await?;
    state.permissions.revoke(PermissionId(perm_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /nodes/{id}/invite`
///
/// Invite a user to collaborate on a node by email.  If the email is already
/// registered in Cognito the permission is granted directly.  If not, a new
/// Cognito user is created (Cognito sends the welcome / temp-password email)
/// and the permission is then granted.
///
/// Requires `Owner` role on the node.  Returns `503` when the Cognito admin
/// client is not configured (i.e. `COGNITO_USER_POOL_ID` is not set).
async fn invite(
    State(state): State<AppState>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<Uuid>,
    Json(req): Json<InviteRequest>,
) -> Result<(StatusCode, Json<Permission>), ApiError> {
    req.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    require_owner(state.permissions.as_ref(), &claims, NodeId(id)).await?;

    let cognito = state.cognito_admin.as_ref().ok_or_else(|| {
        ApiError::Internal("invite requires Cognito admin to be configured".to_string())
    })?;

    // Look up the email in Cognito.  If not found, create the user (which
    // triggers Cognito's welcome email with a temporary password).
    let subject_id = match cognito.find_user_by_email(&req.email).await? {
        Some(user) => user.id,
        None => {
            let new_user = cognito
                .create_user(&common::admin::CreateAdminUserRequest {
                    email: req.email.clone(),
                    first_name: String::new(),
                    last_name: String::new(),
                    initial_roles: vec!["user".to_string()],
                    send_welcome_email: true,
                })
                .await?;
            new_user.id
        }
    };

    let grant_req = GrantPermissionRequest {
        subject_id,
        role: req.role,
    };
    let perm = state
        .permissions
        .grant(NodeId(id), &claims.sub, grant_req)
        .await?;

    Ok((StatusCode::CREATED, Json(perm)))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve wiki-link titles in `body` to node IDs and sync `wiki_link` edges.
async fn sync_wikilinks(
    state: &AppState,
    source_id: NodeId,
    body: &str,
) -> Result<(), ApiError> {
    let titles = parse_wikilink_titles(body);
    let mut target_ids = Vec::new();
    for title in &titles {
        if let Some(id) = state.nodes.find_id_by_title(title).await? {
            target_ids.push(id);
        }
    }
    state.edges.sync_wikilinks(source_id, &target_ids).await?;
    Ok(())
}

/// Strip characters that could inject HTTP headers or break Content-Disposition.
/// Keeps ASCII letters, digits, dots, dashes, underscores, and spaces.
/// Truncates to 200 chars and falls back to "upload" if the result is empty.
fn sanitize_filename(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' '))
        .take(200)
        .collect();
    if clean.trim().is_empty() {
        "upload".to_string()
    } else {
        clean
    }
}
