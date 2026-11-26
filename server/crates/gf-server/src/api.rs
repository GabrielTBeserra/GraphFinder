use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use axum::body::Bytes;
use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Extension, Router};
use gf_core::EmbeddingProvider;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

static HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_REQUEST_DURATION_MICROS_TOTAL: AtomicU64 = AtomicU64::new(0);

pub struct LazyEmbedder {
    cache_dir: String,
    inner: OnceLock<gf_core::FastEmbedProvider>,
}

impl LazyEmbedder {
    pub fn new(cache_dir: String) -> Self {
        Self {
            cache_dir,
            inner: OnceLock::new(),
        }
    }

    pub fn get(&self) -> Result<&gf_core::FastEmbedProvider, String> {
        if let Some(provider) = self.inner.get() {
            return Ok(provider);
        }
        let provider = gf_core::FastEmbedProvider::multilingual_e5_small(self.cache_dir.clone())
            .map_err(|e| e.to_string())?;
        let _ = self.inner.set(provider);
        self.inner
            .get()
            .ok_or_else(|| "embedding provider initialization failed".to_string())
    }
}

pub fn router(
    pool: PgPool,
    embedder: Arc<LazyEmbedder>,
    oidc: Arc<crate::oidc::OidcConfig>,
) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route(
            "/api/v1/auth/oidc/config",
            get(crate::oidc::oidc_public_config),
        )
        .route("/api/v1/auth/oidc/start", get(crate::oidc::oidc_start))
        .route(
            "/api/v1/auth/oidc/callback",
            get(crate::oidc::oidc_callback),
        )
        .route("/api/v1/tokens", get(list_tokens).post(create_token))
        .route("/api/v1/tokens/{id}", axum::routing::delete(revoke_token))
        .route("/api/v1/audit-events", get(list_audit_events))
        .route("/api/v1/users", get(list_users).post(create_user))
        .route(
            "/api/v1/users/{id}/role",
            axum::routing::patch(update_user_role),
        )
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/{id}",
            get(get_project)
                .patch(update_project)
                .delete(delete_project),
        )
        .route(
            "/api/v1/projects/{id}/sources",
            get(list_sources).post(create_source),
        )
        .route(
            "/api/v1/sources/{id}",
            get(get_source).patch(update_source).delete(delete_source),
        )
        .route(
            "/api/v1/sources/{id}/webhook",
            get(get_source_webhook_config)
                .put(upsert_source_webhook_config)
                .delete(delete_source_webhook_config),
        )
        .route("/api/v1/webhooks/{source_id}", post(receive_git_webhook))
        .route(
            "/api/v1/projects/{id}/generations",
            get(list_index_generations),
        )
        .route(
            "/api/v1/projects/{id}/generations/{generation}/rollback",
            post(rollback_index_generation),
        )
        .route(
            "/api/v1/projects/{id}/index-jobs",
            post(create_index_job).get(list_index_jobs),
        )
        .route("/api/v1/index-jobs/{id}", get(get_index_job))
        .route("/api/v1/index-jobs/{id}/cancel", post(cancel_index_job))
        .route(
            "/api/v1/projects/{id}/vault/documents",
            post(upsert_vault_document),
        )
        .route("/api/v1/search", post(search))
        .route("/mcp", post(mcp))
        .layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024))
        .layer(Extension(oidc))
        .layer(Extension(embedder))
        .layer(middleware::from_fn(request_id))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    tenant_slug: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    expires_at: String,
    user_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
}

async fn login(
    State(pool): State<PgPool>,
    axum::Json(request): axum::Json<LoginRequest>,
) -> Result<axum::Json<LoginResponse>, (StatusCode, &'static str)> {
    let rate_bucket = format!(
        "password-login:{}:{}",
        request.tenant_slug.trim().to_lowercase(),
        request.email.trim().to_lowercase()
    );
    let allowed = gf_storage::check_rate_limit(&pool, &rate_bucket, 10, 300)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "rate limit check failed"))?;
    if !allowed {
        return Err((StatusCode::TOO_MANY_REQUESTS, "too many login attempts"));
    }
    let user = gf_storage::find_user_by_email(&pool, &request.tenant_slug, &request.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authentication failed"))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials"))?;
    let hash = user
        .password_hash
        .as_deref()
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials"))?;
    let valid = gf_storage::verify_password(&request.password, hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authentication failed"))?;
    if !valid || user.status != "active" {
        return Err((StatusCode::UNAUTHORIZED, "invalid credentials"));
    }
    let session = gf_storage::create_session(&pool, &user, 12)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authentication failed"))?;
    Ok(axum::Json(LoginResponse {
        token: session.token,
        expires_at: session.expires_at.to_rfc3339(),
        user_id: session.user_id,
        tenant_id: session.tenant_id,
    }))
}

async fn logout(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "missing bearer token"))?;
    gf_storage::revoke_session(&pool, token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "logout failed"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct CreateTokenRequest {
    name: String,
    scopes: Vec<String>,
    project_ids: Option<Vec<uuid::Uuid>>,
    expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct CreateTokenResponse {
    id: uuid::Uuid,
    token: String,
    expires_at: Option<String>,
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

async fn require_principal(
    pool: &PgPool,
    headers: &axum::http::HeaderMap,
) -> Result<gf_storage::Principal, (StatusCode, &'static str)> {
    let token = bearer_token(headers).ok_or((StatusCode::UNAUTHORIZED, "missing bearer token"))?;
    gf_storage::authenticate_bearer_token(pool, token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authentication failed"))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid token"))
}

async fn create_token(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<CreateTokenRequest>,
) -> Result<axum::Json<CreateTokenResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    if principal.kind != "session" {
        return Err((StatusCode::FORBIDDEN, "session authentication required"));
    }
    if !gf_storage::authorize_tenant_scope(&pool, &principal, gf_storage::SCOPE_TOKENS_MANAGE)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authorization failed"))?
    {
        return Err((StatusCode::FORBIDDEN, "insufficient tenant scope"));
    }
    let created = gf_storage::create_api_token(
        &pool,
        principal.tenant_id,
        principal.user_id,
        &request.name,
        request.scopes,
        request.project_ids,
        request.expires_in_days,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "token creation failed"))?;
    audit(&pool,&principal,None,"create","api_token",Some(created.id),serde_json::json!({"name":request.name,"scopes":created.expires_at.map(|v| v.to_rfc3339())})).await;
    Ok(axum::Json(CreateTokenResponse {
        id: created.id,
        token: created.token,
        expires_at: created.expires_at.map(|v| v.to_rfc3339()),
    }))
}

async fn revoke_token(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    if principal.kind != "session" {
        return Err((StatusCode::FORBIDDEN, "session authentication required"));
    }
    if !gf_storage::authorize_tenant_scope(&pool, &principal, gf_storage::SCOPE_TOKENS_MANAGE)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authorization failed"))?
    {
        return Err((StatusCode::FORBIDDEN, "insufficient tenant scope"));
    }
    let revoked = gf_storage::revoke_api_token(&pool, principal.tenant_id, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "token revocation failed"))?;
    if revoked {
        audit(
            &pool,
            &principal,
            None,
            "revoke",
            "api_token",
            Some(id),
            serde_json::json!({}),
        )
        .await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "token not found"))
    }
}

#[derive(Debug, Serialize)]
struct ApiTokenResponse {
    id: uuid::Uuid,
    user_id: Option<uuid::Uuid>,
    name: String,
    scopes: Vec<String>,
    project_ids: Option<Vec<uuid::Uuid>>,
    expires_at: Option<String>,
    revoked_at: Option<String>,
    last_used_at: Option<String>,
    created_at: String,
}

impl From<gf_storage::ApiTokenMetadata> for ApiTokenResponse {
    fn from(token: gf_storage::ApiTokenMetadata) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            name: token.name,
            scopes: token.scopes,
            project_ids: token.project_ids,
            expires_at: token.expires_at.map(|v| v.to_rfc3339()),
            revoked_at: token.revoked_at.map(|v| v.to_rfc3339()),
            last_used_at: token.last_used_at.map(|v| v.to_rfc3339()),
            created_at: token.created_at.to_rfc3339(),
        }
    }
}

async fn require_token_admin(
    pool: &PgPool,
    headers: &axum::http::HeaderMap,
) -> Result<gf_storage::Principal, (StatusCode, &'static str)> {
    let principal = require_principal(pool, headers).await?;
    if principal.kind != "session" {
        return Err((StatusCode::FORBIDDEN, "session authentication required"));
    }
    if !gf_storage::authorize_tenant_scope(pool, &principal, gf_storage::SCOPE_TOKENS_MANAGE)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authorization failed"))?
    {
        return Err((StatusCode::FORBIDDEN, "insufficient tenant scope"));
    }
    Ok(principal)
}

async fn list_tokens(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<Vec<ApiTokenResponse>>, (StatusCode, &'static str)> {
    let principal = require_token_admin(&pool, &headers).await?;
    let tokens = gf_storage::list_api_tokens(&pool, principal.tenant_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "token lookup failed"))?;
    Ok(axum::Json(
        tokens.into_iter().map(ApiTokenResponse::from).collect(),
    ))
}

#[derive(Debug, Serialize)]
struct TenantUserResponse {
    id: uuid::Uuid,
    email: String,
    display_name: String,
    status: String,
    role: Option<String>,
    created_at: String,
}

impl From<gf_storage::TenantUser> for TenantUserResponse {
    fn from(user: gf_storage::TenantUser) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            status: user.status,
            role: user.role,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

async fn list_users(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<Vec<TenantUserResponse>>, (StatusCode, &'static str)> {
    let principal = require_token_admin(&pool, &headers).await?;
    let users = gf_storage::list_tenant_users(&pool, principal.tenant_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "user lookup failed"))?;
    Ok(axum::Json(
        users.into_iter().map(TenantUserResponse::from).collect(),
    ))
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
    display_name: String,
    password: String,
    role: String,
}

fn validate_tenant_role(role: &str) -> Result<(), (StatusCode, &'static str)> {
    if matches!(role, "owner" | "admin" | "member" | "viewer") {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "invalid tenant role"))
    }
}

async fn create_user(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<CreateUserRequest>,
) -> Result<axum::Json<TenantUserResponse>, (StatusCode, &'static str)> {
    let principal = require_token_admin(&pool, &headers).await?;
    validate_tenant_role(&request.role)?;
    let hash = gf_storage::hash_password(&request.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "password hashing failed"))?;
    let user_id = gf_storage::upsert_user(
        &pool,
        principal.tenant_id,
        &request.email,
        &request.display_name,
        &hash,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "user creation failed"))?;
    gf_storage::ensure_tenant_membership(&pool, principal.tenant_id, user_id, &request.role)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership update failed",
            )
        })?;
    let users = gf_storage::list_tenant_users(&pool, principal.tenant_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "user lookup failed"))?;
    let user = users
        .into_iter()
        .find(|user| user.id == user_id)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "user lookup failed"))?;
    audit(
        &pool,
        &principal,
        None,
        "upsert",
        "user",
        Some(user_id),
        serde_json::json!({"email":request.email,"role":request.role}),
    )
    .await;
    Ok(axum::Json(TenantUserResponse::from(user)))
}

#[derive(Debug, Deserialize)]
struct UpdateUserRoleRequest {
    role: String,
}

async fn update_user_role(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<UpdateUserRoleRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_token_admin(&pool, &headers).await?;
    validate_tenant_role(&request.role)?;
    let users = gf_storage::list_tenant_users(&pool, principal.tenant_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "user lookup failed"))?;
    if !users.iter().any(|user| user.id == id) {
        return Err((StatusCode::NOT_FOUND, "user not found"));
    }
    gf_storage::ensure_tenant_membership(&pool, principal.tenant_id, id, &request.role)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership update failed",
            )
        })?;
    audit(
        &pool,
        &principal,
        None,
        "update_role",
        "user",
        Some(id),
        serde_json::json!({"role":request.role}),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct AuditEventResponse {
    id: uuid::Uuid,
    project_id: Option<uuid::Uuid>,
    actor_user_id: Option<uuid::Uuid>,
    api_token_id: Option<uuid::Uuid>,
    action: String,
    resource_type: String,
    resource_id: Option<uuid::Uuid>,
    summary: serde_json::Value,
    created_at: String,
}

impl From<gf_storage::AuditEvent> for AuditEventResponse {
    fn from(event: gf_storage::AuditEvent) -> Self {
        Self {
            id: event.id,
            project_id: event.project_id,
            actor_user_id: event.actor_user_id,
            api_token_id: event.api_token_id,
            action: event.action,
            resource_type: event.resource_type,
            resource_id: event.resource_id,
            summary: event.summary,
            created_at: event.created_at.to_rfc3339(),
        }
    }
}

async fn audit(
    pool: &PgPool,
    principal: &gf_storage::Principal,
    project_id: Option<uuid::Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<uuid::Uuid>,
    summary: serde_json::Value,
) {
    let _ = gf_storage::record_audit_event(
        pool,
        principal,
        project_id,
        action,
        resource_type,
        resource_id,
        summary,
    )
    .await;
}

async fn list_audit_events(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<Vec<AuditEventResponse>>, (StatusCode, &'static str)> {
    let principal = require_token_admin(&pool, &headers).await?;
    let events = gf_storage::list_audit_events(&pool, principal.tenant_id, 100)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "audit lookup failed"))?;
    Ok(axum::Json(
        events.into_iter().map(AuditEventResponse::from).collect(),
    ))
}

#[derive(Debug, Serialize)]
struct ProjectResponse {
    id: uuid::Uuid,
    slug: String,
    name: String,
    description: Option<String>,
    active_generation: i64,
    created_at: String,
    updated_at: String,
}

impl From<gf_storage::models::Project> for ProjectResponse {
    fn from(project: gf_storage::models::Project) -> Self {
        Self {
            id: project.id,
            slug: project.slug,
            name: project.name,
            description: project.description,
            active_generation: project.active_generation,
            created_at: project.created_at.to_rfc3339(),
            updated_at: project.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateProjectRequest {
    slug: String,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateProjectRequest {
    name: Option<String>,
    description: Option<String>,
}

async fn authorize_project_or_error(
    pool: &PgPool,
    principal: &gf_storage::Principal,
    project_id: uuid::Uuid,
    scope: &str,
) -> Result<gf_storage::models::Project, (StatusCode, &'static str)> {
    let project = gf_storage::get_project(pool, project_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project lookup failed"))?
        .filter(|project| project.tenant_id == principal.tenant_id)
        .ok_or((StatusCode::NOT_FOUND, "project not found"))?;
    if !principal.token_project_allowed(project_id) {
        return Err((StatusCode::NOT_FOUND, "project not found"));
    }
    let allowed = gf_storage::authorize_project_scope(pool, principal, project_id, scope)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authorization failed"))?;
    if !allowed {
        return Err((StatusCode::FORBIDDEN, "insufficient project scope"));
    }
    Ok(project)
}

async fn list_projects(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<Vec<ProjectResponse>>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let projects = gf_storage::list_projects(&pool, principal.tenant_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project lookup failed"))?;
    let mut visible = Vec::new();
    for project in projects {
        if gf_storage::authorize_project_scope(
            &pool,
            &principal,
            project.id,
            gf_storage::SCOPE_SEARCH_READ,
        )
        .await
        .unwrap_or(false)
        {
            visible.push(project.into());
        }
    }
    Ok(axum::Json(visible))
}

async fn create_project(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<CreateProjectRequest>,
) -> Result<axum::Json<ProjectResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let allowed =
        gf_storage::authorize_tenant_scope(&pool, &principal, gf_storage::SCOPE_PROJECT_WRITE)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "authorization failed"))?;
    if !allowed {
        return Err((StatusCode::FORBIDDEN, "insufficient tenant scope"));
    }
    let project = gf_storage::create_project(
        &pool,
        gf_storage::models::NewProject {
            tenant_id: principal.tenant_id,
            slug: request.slug,
            name: request.name,
            description: request.description,
            created_by: principal.user_id,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project creation failed"))?;
    if let Some(user_id) = principal.user_id {
        let _ =
            gf_storage::ensure_project_membership(&pool, project.id, user_id, "maintainer").await;
    }
    Ok(axum::Json(project.into()))
}

async fn get_project(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<ProjectResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let project =
        authorize_project_or_error(&pool, &principal, id, gf_storage::SCOPE_SEARCH_READ).await?;
    Ok(axum::Json(project.into()))
}

async fn update_project(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<UpdateProjectRequest>,
) -> Result<axum::Json<ProjectResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let project = gf_storage::update_project(
        &pool,
        id,
        gf_storage::models::ProjectUpdate {
            name: request.name,
            description: request.description,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project update failed"))?
    .ok_or((StatusCode::NOT_FOUND, "project not found"))?;
    audit(
        &pool,
        &principal,
        Some(id),
        "update",
        "project",
        Some(id),
        serde_json::json!({"name":project.name}),
    )
    .await;
    Ok(axum::Json(project.into()))
}

async fn delete_project(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let deleted = gf_storage::delete_project(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project deletion failed"))?;
    if deleted {
        audit(
            &pool,
            &principal,
            Some(id),
            "delete",
            "project",
            Some(id),
            serde_json::json!({}),
        )
        .await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "project not found"))
    }
}

#[derive(Debug, Serialize)]
struct SourceResponse {
    id: uuid::Uuid,
    project_id: uuid::Uuid,
    kind: String,
    uri: String,
    branch: Option<String>,
    credential_ref: Option<String>,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    config: serde_json::Value,
    auto_index_interval_seconds: Option<i32>,
    next_index_at: Option<String>,
    created_at: String,
    updated_at: String,
    last_indexed_commit: Option<String>,
    indexing_status: String,
}

impl From<gf_storage::models::Source> for SourceResponse {
    fn from(source: gf_storage::models::Source) -> Self {
        Self {
            id: source.id,
            project_id: source.project_id,
            kind: source.kind,
            uri: source.uri,
            branch: source.branch,
            credential_ref: source.credential_ref,
            include_patterns: source.include_patterns,
            exclude_patterns: source.exclude_patterns,
            config: source.config,
            auto_index_interval_seconds: source.auto_index_interval_seconds,
            next_index_at: source.next_index_at.map(|v| v.to_rfc3339()),
            created_at: source.created_at.to_rfc3339(),
            updated_at: source.updated_at.to_rfc3339(),
            last_indexed_commit: None,
            indexing_status: "never_indexed".into(),
        }
    }
}

async fn source_response(pool: &PgPool, source: gf_storage::models::Source) -> SourceResponse {
    let source_id = source.id;
    let mut response = SourceResponse::from(source);
    if let Some((revisions, status)) = sqlx::query_as::<_, (serde_json::Value, String)>(
        r#"
        SELECT g.source_revisions,g.status
        FROM projects p
        JOIN index_generations g ON g.project_id=p.id AND g.generation=p.active_generation
        WHERE p.id=$1
        "#,
    )
    .bind(response.project_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    {
        response.last_indexed_commit = revisions
            .get(source_id.to_string())
            .and_then(|value| value.as_str())
            .map(str::to_string);
        response.indexing_status = status;
    }
    response
}

#[derive(Debug, Deserialize)]
struct CreateSourceRequest {
    kind: String,
    uri: String,
    branch: Option<String>,
    credential_ref: Option<String>,
    include_patterns: Option<Vec<String>>,
    exclude_patterns: Option<Vec<String>>,
    config: Option<serde_json::Value>,
    auto_index_interval_seconds: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct UpdateSourceRequest {
    uri: Option<String>,
    branch: Option<String>,
    credential_ref: Option<String>,
    include_patterns: Option<Vec<String>>,
    exclude_patterns: Option<Vec<String>>,
    config: Option<serde_json::Value>,
    auto_index_interval_seconds: Option<Option<i32>>,
}

fn validate_source_uri(kind: &str, uri: &str) -> Result<String, (StatusCode, &'static str)> {
    if kind == "managed_vault" {
        return Ok(uri.to_string());
    }
    if kind != "git" {
        return Err((StatusCode::BAD_REQUEST, "unsupported source kind"));
    }
    gf_indexer::validate_git_source_url(uri, &gf_indexer::GitSourcePolicy::default())
        .map(|url| url.normalized_url)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid or unsafe git source url"))
}

async fn authorize_source_or_error(
    pool: &PgPool,
    principal: &gf_storage::Principal,
    source_id: uuid::Uuid,
    scope: &str,
) -> Result<gf_storage::models::Source, (StatusCode, &'static str)> {
    let source = gf_storage::get_source(pool, source_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "source not found"))?;
    authorize_project_or_error(pool, principal, source.project_id, scope).await?;
    Ok(source)
}

async fn list_sources(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<Vec<SourceResponse>>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(&pool, &principal, project_id, gf_storage::SCOPE_SEARCH_READ)
        .await?;
    let sources = gf_storage::list_sources(&pool, project_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source lookup failed"))?;
    let mut responses = Vec::new();
    for source in sources {
        responses.push(source_response(&pool, source).await);
    }
    Ok(axum::Json(responses))
}

async fn create_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<CreateSourceRequest>,
) -> Result<axum::Json<SourceResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(
        &pool,
        &principal,
        project_id,
        gf_storage::SCOPE_PROJECT_WRITE,
    )
    .await?;
    let uri = validate_source_uri(&request.kind, &request.uri)?;
    let source = gf_storage::create_source(
        &pool,
        gf_storage::models::NewSource {
            project_id,
            kind: request.kind,
            uri,
            branch: request.branch,
            credential_ref: request.credential_ref,
            include_patterns: request.include_patterns.unwrap_or_default(),
            exclude_patterns: request.exclude_patterns.unwrap_or_default(),
            config: request.config.unwrap_or_else(|| serde_json::json!({})),
            auto_index_interval_seconds: request.auto_index_interval_seconds,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source creation failed"))?;
    audit(
        &pool,
        &principal,
        Some(project_id),
        "create",
        "source",
        Some(source.id),
        serde_json::json!({"kind":source.kind,"uri":source.uri}),
    )
    .await;
    Ok(axum::Json(source_response(&pool, source).await))
}

async fn get_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<SourceResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_SEARCH_READ).await?;
    Ok(axum::Json(source_response(&pool, source).await))
}

async fn update_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<UpdateSourceRequest>,
) -> Result<axum::Json<SourceResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let uri = request
        .uri
        .map(|uri| validate_source_uri(&source.kind, &uri))
        .transpose()?;
    let updated = gf_storage::update_source(
        &pool,
        id,
        gf_storage::models::SourceUpdate {
            uri,
            branch: request.branch,
            credential_ref: request.credential_ref,
            include_patterns: request.include_patterns,
            exclude_patterns: request.exclude_patterns,
            config: request.config,
            auto_index_interval_seconds: request.auto_index_interval_seconds,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source update failed"))?
    .ok_or((StatusCode::NOT_FOUND, "source not found"))?;
    audit(
        &pool,
        &principal,
        Some(updated.project_id),
        "update",
        "source",
        Some(updated.id),
        serde_json::json!({"uri":updated.uri}),
    )
    .await;
    Ok(axum::Json(source_response(&pool, updated).await))
}

async fn delete_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let deleted = gf_storage::delete_source(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source deletion failed"))?;
    if deleted {
        audit(
            &pool,
            &principal,
            Some(source.project_id),
            "delete",
            "source",
            Some(id),
            serde_json::json!({}),
        )
        .await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "source not found"))
    }
}

#[derive(Debug, Deserialize)]
struct UpsertWebhookRequest {
    provider: String,
    secret: String,
    enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
struct WebhookConfigResponse {
    source_id: uuid::Uuid,
    project_id: uuid::Uuid,
    provider: String,
    enabled: bool,
    endpoint_path: String,
    last_delivery_at: Option<String>,
    last_status: Option<String>,
    created_at: String,
    updated_at: String,
}

fn webhook_config_response(webhook: gf_storage::SourceWebhook) -> WebhookConfigResponse {
    WebhookConfigResponse {
        source_id: webhook.source_id,
        project_id: webhook.project_id,
        provider: webhook.provider,
        enabled: webhook.enabled,
        endpoint_path: format!("/api/v1/webhooks/{}", webhook.source_id),
        last_delivery_at: webhook.last_delivery_at.map(|v| v.to_rfc3339()),
        last_status: webhook.last_status,
        created_at: webhook.created_at.to_rfc3339(),
        updated_at: webhook.updated_at.to_rfc3339(),
    }
}

fn validate_webhook_provider(provider: &str) -> Result<(), (StatusCode, &'static str)> {
    match provider {
        "github" | "gitlab" | "generic" => Ok(()),
        _ => Err((StatusCode::BAD_REQUEST, "unsupported webhook provider")),
    }
}

async fn upsert_source_webhook_config(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<UpsertWebhookRequest>,
) -> Result<axum::Json<WebhookConfigResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    if source.kind != "git" {
        return Err((
            StatusCode::BAD_REQUEST,
            "webhooks are only supported for git sources",
        ));
    }
    validate_webhook_provider(&request.provider)?;
    if request.secret.len() < 16 {
        return Err((
            StatusCode::BAD_REQUEST,
            "webhook secret must contain at least 16 characters",
        ));
    }
    let webhook = gf_storage::upsert_source_webhook(
        &pool,
        source.id,
        source.project_id,
        &request.provider,
        &request.secret,
        request.enabled.unwrap_or(true),
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "webhook configuration failed",
        )
    })?;
    audit(
        &pool,
        &principal,
        Some(source.project_id),
        "configure",
        "source_webhook",
        Some(source.id),
        serde_json::json!({"provider":webhook.provider,"enabled":webhook.enabled}),
    )
    .await;
    Ok(axum::Json(webhook_config_response(webhook)))
}

async fn get_source_webhook_config(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<WebhookConfigResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_SEARCH_READ).await?;
    let webhook = gf_storage::get_source_webhook(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "webhook lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "webhook not configured"))?;
    Ok(axum::Json(webhook_config_response(webhook)))
}

async fn delete_source_webhook_config(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let deleted = gf_storage::delete_source_webhook(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "webhook deletion failed"))?;
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "webhook not configured"));
    }
    audit(
        &pool,
        &principal,
        Some(source.project_id),
        "delete",
        "source_webhook",
        Some(source.id),
        serde_json::json!({}),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

type HmacSha256 = Hmac<Sha256>;

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(value.get(index..index + 2)?, 16).ok())
        .collect()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn verify_hmac_signature(secret: &str, body: &[u8], header: Option<&str>) -> bool {
    let Some(header) = header else {
        return false;
    };
    let encoded = header.strip_prefix("sha256=").unwrap_or(header);
    let Some(expected) = decode_hex(encoded) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

fn validate_webhook_delivery(
    provider: &str,
    secret: &str,
    headers: &axum::http::HeaderMap,
    body: &[u8],
) -> bool {
    match provider {
        "github" => verify_hmac_signature(
            secret,
            body,
            headers
                .get("x-hub-signature-256")
                .and_then(|value| value.to_str().ok()),
        ),
        "gitlab" => {
            let Some(token) = headers
                .get("x-gitlab-token")
                .and_then(|value| value.to_str().ok())
            else {
                return false;
            };
            constant_time_eq(token.as_bytes(), secret.as_bytes())
        }
        "generic" => verify_hmac_signature(
            secret,
            body,
            headers
                .get("x-graphfinder-signature")
                .and_then(|value| value.to_str().ok()),
        ),
        _ => false,
    }
}

fn normalize_repository_url(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    trimmed.strip_suffix(".git").unwrap_or(trimmed).to_string()
}

fn repository_urls(payload: &serde_json::Value) -> Vec<String> {
    [
        "/repository/clone_url",
        "/repository/git_url",
        "/repository/ssh_url",
        "/repository/html_url",
        "/project/git_http_url",
        "/project/git_ssh_url",
        "/project/web_url",
        "/repository/url",
    ]
    .iter()
    .filter_map(|pointer| payload.pointer(pointer)?.as_str().map(str::to_string))
    .collect()
}

fn repository_matches(source_uri: &str, payload: &serde_json::Value) -> bool {
    let urls = repository_urls(payload);
    if urls.is_empty() {
        return true;
    }
    let source = normalize_repository_url(source_uri);
    let source_lower = source.to_ascii_lowercase();
    urls.iter().any(|candidate| {
        let candidate = normalize_repository_url(candidate);
        candidate == source || candidate.to_ascii_lowercase() == source_lower
    })
}

fn payload_branch(payload: &serde_json::Value) -> Option<String> {
    let reference = payload.get("ref")?.as_str()?;
    if let Some(branch) = reference.strip_prefix("refs/heads/") {
        return Some(branch.to_string());
    }
    if reference.starts_with("refs/") {
        return None;
    }
    Some(reference.to_string())
}

fn is_zero_revision(value: Option<&serde_json::Value>) -> bool {
    value
        .and_then(|value| value.as_str())
        .is_some_and(|value| !value.is_empty() && value.chars().all(|ch| ch == '0'))
}

#[derive(Debug, Serialize)]
struct WebhookDeliveryResponse {
    status: &'static str,
    job_id: Option<uuid::Uuid>,
    reason: Option<&'static str>,
}

async fn record_webhook_audit(
    pool: &PgPool,
    project_id: uuid::Uuid,
    source_id: uuid::Uuid,
    summary: serde_json::Value,
) {
    let tenant_id =
        sqlx::query_scalar::<_, uuid::Uuid>("SELECT tenant_id FROM projects WHERE id=$1")
            .bind(project_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    if let Some(tenant_id) = tenant_id {
        let _ = sqlx::query(
            r#"
            INSERT INTO audit_events (tenant_id,project_id,actor_user_id,api_token_id,action,resource_type,resource_id,summary)
            VALUES ($1,$2,NULL,NULL,'webhook','source',$3,$4)
            "#,
        )
        .bind(tenant_id)
        .bind(project_id)
        .bind(source_id)
        .bind(summary)
        .execute(pool)
        .await;
    }
}

async fn receive_git_webhook(
    State(pool): State<PgPool>,
    axum::extract::Path(source_id): axum::extract::Path<uuid::Uuid>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<axum::Json<WebhookDeliveryResponse>, (StatusCode, &'static str)> {
    let webhook = gf_storage::get_source_webhook(&pool, source_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "webhook lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "webhook not configured"))?;
    if !validate_webhook_delivery(&webhook.provider, &webhook.secret, &headers, &body) {
        return Err((StatusCode::UNAUTHORIZED, "invalid webhook signature"));
    }
    if !webhook.enabled {
        return Err((StatusCode::FORBIDDEN, "webhook is disabled"));
    }
    let source = gf_storage::get_source(&pool, source_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "source not found"))?;
    if source.kind != "git" {
        return Err((
            StatusCode::BAD_REQUEST,
            "webhooks are only supported for git sources",
        ));
    }
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid webhook payload"))?;

    let Some(branch) = payload_branch(&payload) else {
        let _ = gf_storage::record_source_webhook_delivery(&pool, source_id, "ignored").await;
        return Ok(axum::Json(WebhookDeliveryResponse {
            status: "ignored",
            job_id: None,
            reason: Some("not_a_branch_push"),
        }));
    };
    if let Some(configured_branch) = source.branch.as_deref() {
        if configured_branch != branch {
            let _ = gf_storage::record_source_webhook_delivery(&pool, source_id, "ignored").await;
            return Ok(axum::Json(WebhookDeliveryResponse {
                status: "ignored",
                job_id: None,
                reason: Some("branch_mismatch"),
            }));
        }
    }
    if is_zero_revision(payload.get("after")) {
        let _ = gf_storage::record_source_webhook_delivery(&pool, source_id, "ignored").await;
        return Ok(axum::Json(WebhookDeliveryResponse {
            status: "ignored",
            job_id: None,
            reason: Some("branch_deleted"),
        }));
    }
    if !repository_matches(&source.uri, &payload) {
        let _ = gf_storage::record_source_webhook_delivery(&pool, source_id, "rejected").await;
        return Err((
            StatusCode::BAD_REQUEST,
            "webhook repository does not match source",
        ));
    }

    let job_payload = serde_json::json!({
        "trigger": "webhook",
        "incremental": true,
        "provider": webhook.provider,
        "ref": payload.get("ref").cloned().unwrap_or(serde_json::Value::Null),
        "branch": branch,
        "before": payload.get("before").cloned().unwrap_or(serde_json::Value::Null),
        "after": payload.get("after").cloned().unwrap_or(serde_json::Value::Null),
        "head_commit": payload.pointer("/head_commit/id").cloned().unwrap_or(serde_json::Value::Null)
    });
    let job = gf_storage::enqueue_source_index_job_if_idle(
        &pool,
        gf_storage::NewIndexJob {
            project_id: source.project_id,
            source_id: Some(source.id),
            priority: 10,
            max_attempts: 3,
            payload: job_payload.clone(),
            created_by: None,
        },
    )
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "webhook job enqueue failed",
        )
    })?;

    let (status, job_id, reason) = if let Some(job) = job {
        ("queued", Some(job.id), None)
    } else {
        ("duplicate", None, Some("source_already_has_active_job"))
    };
    let _ = gf_storage::record_source_webhook_delivery(&pool, source_id, status).await;
    record_webhook_audit(
        &pool,
        source.project_id,
        source.id,
        serde_json::json!({
            "status":status,
            "job_id":job_id,
            "payload":job_payload,
        }),
    )
    .await;
    Ok(axum::Json(WebhookDeliveryResponse {
        status,
        job_id,
        reason,
    }))
}

async fn request_id(mut request: Request, next: Next) -> Response {
    let started = Instant::now();
    let value = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if let Ok(header) = HeaderValue::from_str(&value) {
        request.headers_mut().insert("x-request-id", header.clone());
        let mut response = next.run(request).await;
        HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
        HTTP_REQUEST_DURATION_MICROS_TOTAL
            .fetch_add(started.elapsed().as_micros() as u64, Ordering::Relaxed);
        response.headers_mut().insert("x-request-id", header);
        response
    } else {
        let response = next.run(request).await;
        HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
        HTTP_REQUEST_DURATION_MICROS_TOTAL
            .fetch_add(started.elapsed().as_micros() as u64, Ordering::Relaxed);
        response
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(State(pool): State<PgPool>) -> Result<&'static str, impl IntoResponse> {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(1) => Ok("ready"),
        _ => Err((StatusCode::SERVICE_UNAVAILABLE, "database is not ready")),
    }
}

async fn metrics(State(pool): State<PgPool>) -> String {
    let job_metrics = sqlx::query_as::<_, (i64, i64, i64, i64, f64)>(
        r#"
        SELECT
          count(*) FILTER (WHERE status='queued'),
          count(*) FILTER (WHERE status='running'),
          count(*) FILTER (WHERE status='failed'),
          count(*) FILTER (WHERE status='completed'),
          coalesce(avg(extract(epoch from finished_at-started_at)) FILTER (WHERE status='completed' AND started_at IS NOT NULL AND finished_at IS NOT NULL),0)::float8
        FROM index_jobs
        "#,
    )
    .fetch_one(&pool)
    .await
    .unwrap_or((0, 0, 0, 0, 0.0));
    let chunks_total = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM chunks")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let embeddings_total = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM chunk_embeddings")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let requests = HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed);
    let duration_seconds =
        HTTP_REQUEST_DURATION_MICROS_TOTAL.load(Ordering::Relaxed) as f64 / 1_000_000.0;
    format!(
        concat!(
            "# HELP graphfinder_server_up Whether the API process is running.\n",
            "# TYPE graphfinder_server_up gauge\n",
            "graphfinder_server_up 1\n",
            "# HELP graphfinder_http_requests_total HTTP requests handled by this process.\n",
            "# TYPE graphfinder_http_requests_total counter\n",
            "graphfinder_http_requests_total {requests}\n",
            "# HELP graphfinder_http_request_duration_seconds_sum Total HTTP request duration in seconds.\n",
            "# TYPE graphfinder_http_request_duration_seconds_sum counter\n",
            "graphfinder_http_request_duration_seconds_sum {duration_seconds:.6}\n",
            "# HELP graphfinder_index_jobs Index jobs by status.\n",
            "# TYPE graphfinder_index_jobs gauge\n",
            "graphfinder_index_jobs{{status=\"queued\"}} {queued}\n",
            "graphfinder_index_jobs{{status=\"running\"}} {running}\n",
            "graphfinder_index_jobs{{status=\"failed\"}} {failed}\n",
            "graphfinder_index_jobs{{status=\"completed\"}} {completed}\n",
            "# HELP graphfinder_index_job_duration_seconds_avg Average completed indexing job duration.\n",
            "# TYPE graphfinder_index_job_duration_seconds_avg gauge\n",
            "graphfinder_index_job_duration_seconds_avg {job_duration:.6}\n",
            "# HELP graphfinder_chunks_total Indexed chunks.\n",
            "# TYPE graphfinder_chunks_total gauge\n",
            "graphfinder_chunks_total {chunks_total}\n",
            "# HELP graphfinder_embeddings_total Persisted chunk embeddings.\n",
            "# TYPE graphfinder_embeddings_total gauge\n",
            "graphfinder_embeddings_total {embeddings_total}\n"
        ),
        requests = requests,
        duration_seconds = duration_seconds,
        queued = job_metrics.0,
        running = job_metrics.1,
        failed = job_metrics.2,
        completed = job_metrics.3,
        job_duration = job_metrics.4,
        chunks_total = chunks_total,
        embeddings_total = embeddings_total,
    )
}

#[derive(Debug, Serialize)]
struct IndexJobResponse {
    id: uuid::Uuid,
    project_id: uuid::Uuid,
    source_id: Option<uuid::Uuid>,
    generation: Option<i64>,
    status: String,
    priority: i32,
    attempts: i32,
    max_attempts: i32,
    payload: serde_json::Value,
    error: Option<String>,
    progress_percent: i32,
    current_stage: Option<String>,
    run_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<gf_storage::IndexJob> for IndexJobResponse {
    fn from(job: gf_storage::IndexJob) -> Self {
        Self {
            id: job.id,
            project_id: job.project_id,
            source_id: job.source_id,
            generation: job.generation,
            status: job.status,
            priority: job.priority,
            attempts: job.attempts,
            max_attempts: job.max_attempts,
            payload: job.payload,
            error: job.error,
            progress_percent: job.progress_percent,
            current_stage: job.current_stage,
            run_at: job.run_at.to_rfc3339(),
            started_at: job.started_at.map(|v| v.to_rfc3339()),
            finished_at: job.finished_at.map(|v| v.to_rfc3339()),
            created_at: job.created_at.to_rfc3339(),
            updated_at: job.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
struct IndexGenerationResponse {
    project_id: uuid::Uuid,
    generation: i64,
    status: String,
    embedding_model: String,
    embedding_dimensions: i32,
    source_revisions: serde_json::Value,
    error: Option<String>,
    is_active: bool,
    created_at: String,
    started_at: Option<String>,
    published_at: Option<String>,
}

fn generation_response(
    generation: gf_storage::models::IndexGeneration,
    active_generation: i64,
) -> IndexGenerationResponse {
    IndexGenerationResponse {
        project_id: generation.project_id,
        generation: generation.generation,
        status: generation.status,
        embedding_model: generation.embedding_model,
        embedding_dimensions: generation.embedding_dimensions,
        source_revisions: generation.source_revisions,
        error: generation.error,
        is_active: generation.generation == active_generation,
        created_at: generation.created_at.to_rfc3339(),
        started_at: generation.started_at.map(|value| value.to_rfc3339()),
        published_at: generation.published_at.map(|value| value.to_rfc3339()),
    }
}

async fn list_index_generations(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<Vec<IndexGenerationResponse>>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let project =
        authorize_project_or_error(&pool, &principal, project_id, gf_storage::SCOPE_SEARCH_READ)
            .await?;
    let generations = gf_storage::list_index_generations(&pool, project_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "generation lookup failed",
            )
        })?
        .into_iter()
        .map(|generation| generation_response(generation, project.active_generation))
        .collect();
    Ok(axum::Json(generations))
}

#[derive(Debug, Deserialize)]
struct RollbackGenerationRequest {
    confirm_generation: i64,
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct RollbackGenerationResponse {
    project_id: uuid::Uuid,
    active_generation: i64,
    status: &'static str,
}

async fn rollback_index_generation(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path((project_id, generation)): axum::extract::Path<(uuid::Uuid, i64)>,
    axum::Json(request): axum::Json<RollbackGenerationRequest>,
) -> Result<axum::Json<RollbackGenerationResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let project = authorize_project_or_error(
        &pool,
        &principal,
        project_id,
        gf_storage::SCOPE_PROJECT_WRITE,
    )
    .await?;
    if request.confirm_generation != generation {
        return Err((
            StatusCode::BAD_REQUEST,
            "confirmation does not match target generation",
        ));
    }
    if generation == project.active_generation {
        return Err((
            StatusCode::BAD_REQUEST,
            "rollback target is already the active generation",
        ));
    }
    let has_active_jobs = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM index_jobs WHERE project_id=$1 AND status IN ('queued','running'))",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "job state lookup failed"))?;
    if has_active_jobs {
        return Err((
            StatusCode::CONFLICT,
            "cancel or finish active indexing jobs before rollback",
        ));
    }
    let activated = gf_storage::activate_index_generation(&pool, project_id, generation)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "generation rollback failed",
            )
        })?;
    if !activated {
        return Err((
            StatusCode::CONFLICT,
            "target generation is not rollback eligible",
        ));
    }
    audit(
        &pool,
        &principal,
        Some(project_id),
        "rollback",
        "index_generation",
        None,
        serde_json::json!({
            "from_generation":project.active_generation,
            "to_generation":generation,
            "reason":request.reason,
        }),
    )
    .await;
    Ok(axum::Json(RollbackGenerationResponse {
        project_id,
        active_generation: generation,
        status: "rolled_back",
    }))
}

#[derive(Debug, Deserialize)]
struct CreateIndexJobRequest {
    source_id: Option<uuid::Uuid>,
    priority: Option<i32>,
    max_attempts: Option<i32>,
    payload: Option<serde_json::Value>,
}

async fn create_index_job(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<CreateIndexJobRequest>,
) -> Result<axum::Json<IndexJobResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(
        &pool,
        &principal,
        project_id,
        gf_storage::SCOPE_INDEX_TRIGGER,
    )
    .await?;
    if let Some(source_id) = request.source_id {
        let source = gf_storage::get_source(&pool, source_id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source lookup failed"))?
            .ok_or((StatusCode::NOT_FOUND, "source not found"))?;
        if source.project_id != project_id {
            return Err((StatusCode::BAD_REQUEST, "source does not belong to project"));
        }
    }
    let job = gf_storage::enqueue_index_job(
        &pool,
        gf_storage::NewIndexJob {
            project_id,
            source_id: request.source_id,
            priority: request.priority.unwrap_or(0),
            max_attempts: request.max_attempts.unwrap_or(3),
            payload: request.payload.unwrap_or_else(|| serde_json::json!({})),
            created_by: principal.user_id,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "job creation failed"))?;
    audit(
        &pool,
        &principal,
        Some(project_id),
        "create",
        "index_job",
        Some(job.id),
        serde_json::json!({"source_id":job.source_id,"priority":job.priority}),
    )
    .await;
    Ok(axum::Json(job.into()))
}

async fn list_index_jobs(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<Vec<IndexJobResponse>>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(&pool, &principal, project_id, gf_storage::SCOPE_SEARCH_READ)
        .await?;
    let jobs = gf_storage::list_index_jobs(&pool, project_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "job lookup failed"))?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(axum::Json(jobs))
}

async fn authorize_job_or_error(
    pool: &PgPool,
    principal: &gf_storage::Principal,
    job_id: uuid::Uuid,
    scope: &str,
) -> Result<gf_storage::IndexJob, (StatusCode, &'static str)> {
    let job = gf_storage::get_index_job(pool, job_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "job lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "job not found"))?;
    authorize_project_or_error(pool, principal, job.project_id, scope).await?;
    Ok(job)
}

async fn get_index_job(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<IndexJobResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let job = authorize_job_or_error(&pool, &principal, id, gf_storage::SCOPE_SEARCH_READ).await?;
    Ok(axum::Json(job.into()))
}

async fn cancel_index_job(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_job_or_error(&pool, &principal, id, gf_storage::SCOPE_INDEX_TRIGGER).await?;
    let cancelled = gf_storage::cancel_index_job(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "job cancellation failed"))?;
    if cancelled {
        audit(
            &pool,
            &principal,
            None,
            "cancel",
            "index_job",
            Some(id),
            serde_json::json!({}),
        )
        .await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::CONFLICT, "job is not cancellable"))
    }
}

async fn mcp(
    State(pool): State<PgPool>,
    Extension(embedder): Extension<Arc<LazyEmbedder>>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> Result<Response, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let is_tool_call = request.get("method").and_then(|v| v.as_str()) == Some("tools/call");
    let embedder_ref: Option<&dyn EmbeddingProvider> = if is_tool_call {
        let warm = embedder.clone();
        tokio::task::spawn_blocking(move || warm.get().map(|_| ()).map_err(|e| e.to_string()))
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "embedding task failed"))?
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "embedding initialization failed",
                )
            })?;
        let provider: &dyn EmbeddingProvider = embedder.get().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "embedding initialization failed",
            )
        })?;
        Some(provider)
    } else {
        None
    };
    let context = gf_mcp::McpContext {
        pool: &pool,
        principal: &principal,
        embedder: embedder_ref,
    };
    let Some(response_value) = gf_mcp::handle_rpc(&context, request).await else {
        return Ok(StatusCode::ACCEPTED.into_response());
    };
    let mut response = axum::Json(response_value).into_response();
    if let Ok(value) = HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()) {
        response.headers_mut().insert("mcp-session-id", value);
    }
    Ok(response)
}

#[derive(Debug, Deserialize)]
struct UpsertVaultDocumentRequest {
    source_id: uuid::Uuid,
    path: String,
    content: String,
    expected_revision: Option<i64>,
}

#[derive(Debug, Serialize)]
struct VaultDocumentResponse {
    id: uuid::Uuid,
    project_id: uuid::Uuid,
    generation: i64,
    source_id: Option<uuid::Uuid>,
    path: String,
    title: Option<String>,
    revision: i64,
    updated_at: String,
}

impl From<gf_storage::VaultDocumentRecord> for VaultDocumentResponse {
    fn from(document: gf_storage::VaultDocumentRecord) -> Self {
        Self {
            id: document.id,
            project_id: document.project_id,
            generation: document.generation,
            source_id: document.source_id,
            path: document.path,
            title: document.title,
            revision: document.revision,
            updated_at: document.updated_at.to_rfc3339(),
        }
    }
}

async fn upsert_vault_document(
    State(pool): State<PgPool>,
    Extension(embedder): Extension<Arc<LazyEmbedder>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(project_id): axum::extract::Path<uuid::Uuid>,
    axum::Json(request): axum::Json<UpsertVaultDocumentRequest>,
) -> Result<axum::Json<VaultDocumentResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(
        &pool,
        &principal,
        project_id,
        gf_storage::SCOPE_PROJECT_WRITE,
    )
    .await?;
    let project = gf_storage::get_project(&pool, project_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "project lookup failed"))?
        .ok_or((StatusCode::NOT_FOUND, "project not found"))?;
    let note = gf_core::parse_markdown(
        request
            .path
            .trim_end_matches(".md")
            .rsplit('/')
            .next()
            .unwrap_or("note"),
        &request.content,
    );
    let passages = note
        .chunks
        .iter()
        .map(|chunk| chunk.content.clone())
        .collect::<Vec<_>>();
    let embeddings = tokio::task::spawn_blocking(move || {
        embedder
            .get()
            .map_err(|e| e.to_string())?
            .embed_passages(&passages)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "embedding task failed"))?
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding generation failed",
        )
    })?;
    if embeddings.len() != note.chunks.len() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding count mismatch",
        ));
    }
    let chunks = note
        .chunks
        .iter()
        .zip(embeddings)
        .map(|(chunk, embedding)| gf_storage::VaultChunkWrite {
            chunk_index: chunk.chunk_index as i32,
            heading_path: chunk.heading.iter().cloned().collect(),
            content: chunk.content.clone(),
            token_count: chunk.token_estimate as i32,
            content_hash: gf_core::content_hash(&chunk.content),
            metadata: serde_json::json!({"links":chunk.links,"tags":chunk.tags}),
            embedding,
        })
        .collect();
    let write = gf_storage::VaultDocumentWrite {
        project_id,
        generation: project.active_generation,
        source_id: request.source_id,
        path: request.path,
        title: note.title,
        content_hash: gf_core::content_hash(&request.content),
        size_bytes: request.content.len() as i64,
        metadata: serde_json::json!({"summary":note.summary,"links":note.links,"tags":note.tags,"aliases":note.aliases,"content":request.content}),
        chunks,
    };
    let document = gf_storage::upsert_vault_document(&pool, write, request.expected_revision)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "vault document write failed");
            match error {
                gf_storage::StorageError::Conflict(_) => (
                    StatusCode::CONFLICT,
                    "vault document conflict or readonly source",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "vault document write failed",
                ),
            }
        })?;
    audit(
        &pool,
        &principal,
        Some(project_id),
        "upsert",
        "vault_document",
        Some(document.id),
        serde_json::json!({"path":document.path,"revision":document.revision}),
    )
    .await;
    Ok(axum::Json(VaultDocumentResponse::from(document)))
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    project_id: uuid::Uuid,
    query: String,
    mode: Option<String>,
    language: Option<String>,
    document_type: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SearchHitResponse {
    chunk_id: uuid::Uuid,
    document_id: uuid::Uuid,
    path: String,
    heading_path: Vec<String>,
    content: String,
    score: f32,
}

impl From<gf_storage::SearchHit> for SearchHitResponse {
    fn from(hit: gf_storage::SearchHit) -> Self {
        Self {
            chunk_id: hit.chunk_id,
            document_id: hit.document_id,
            path: hit.path,
            heading_path: hit.heading_path,
            content: hit.content,
            score: hit.score,
        }
    }
}

async fn search(
    State(pool): State<PgPool>,
    Extension(embedder): Extension<Arc<LazyEmbedder>>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<SearchRequest>,
) -> Result<axum::Json<Vec<SearchHitResponse>>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_project_or_error(
        &pool,
        &principal,
        request.project_id,
        gf_storage::SCOPE_SEARCH_READ,
    )
    .await?;
    let filters = gf_storage::SearchFilters {
        language: request.language,
        document_type: request.document_type,
    };
    let limit = request.limit.unwrap_or(10).clamp(1, 100);
    let mode = request.mode.as_deref().unwrap_or("hybrid");
    let hits = if mode == "lexical" {
        gf_storage::lexical_search(&pool, request.project_id, &request.query, &filters, limit).await
    } else {
        let query = request.query.clone();
        let embedding = tokio::task::spawn_blocking(move || {
            embedder
                .get()
                .map_err(|e| e.to_string())?
                .embed_query(&query)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "embedding task failed"))?
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "embedding generation failed",
            )
        })?;
        if mode == "vector" {
            gf_storage::vector_search(&pool, request.project_id, &embedding, &filters, limit).await
        } else {
            gf_storage::hybrid_search(
                &pool,
                request.project_id,
                &request.query,
                &embedding,
                &filters,
                limit,
            )
            .await
        }
    }
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "search failed"))?;
    Ok(axum::Json(
        hits.into_iter().map(Into::into).collect::<Vec<_>>(),
    ))
}
