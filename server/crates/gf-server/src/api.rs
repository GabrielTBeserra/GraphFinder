use std::sync::{Arc, OnceLock};

use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Extension, Router};
use gf_core::EmbeddingProvider;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

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

pub fn router(pool: PgPool, embedder: Arc<LazyEmbedder>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/tokens", get(list_tokens).post(create_token))
        .route("/api/v1/tokens/{id}", axum::routing::delete(revoke_token))
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
            "/api/v1/projects/{id}/index-jobs",
            post(create_index_job).get(list_index_jobs),
        )
        .route("/api/v1/index-jobs/{id}", get(get_index_job))
        .route("/api/v1/index-jobs/{id}/cancel", post(cancel_index_job))
        .route("/api/v1/search", post(search))
        .route("/mcp", post(mcp))
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
    Ok(StatusCode::NO_CONTENT)
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
}

#[derive(Debug, Deserialize)]
struct UpdateSourceRequest {
    uri: Option<String>,
    branch: Option<String>,
    credential_ref: Option<String>,
    include_patterns: Option<Vec<String>>,
    exclude_patterns: Option<Vec<String>>,
    config: Option<serde_json::Value>,
}

fn validate_source_uri(kind: &str, uri: &str) -> Result<String, (StatusCode, &'static str)> {
    if kind != "git" {
        return Ok(uri.to_string());
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
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source creation failed"))?;
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
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source update failed"))?
    .ok_or((StatusCode::NOT_FOUND, "source not found"))?;
    Ok(axum::Json(source_response(&pool, updated).await))
}

async fn delete_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_PROJECT_WRITE).await?;
    let deleted = gf_storage::delete_source(&pool, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source deletion failed"))?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "source not found"))
    }
}

async fn request_id(mut request: Request, next: Next) -> Response {
    let value = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if let Ok(header) = HeaderValue::from_str(&value) {
        request.headers_mut().insert("x-request-id", header.clone());
        let mut response = next.run(request).await;
        response.headers_mut().insert("x-request-id", header);
        response
    } else {
        next.run(request).await
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

async fn metrics() -> &'static str {
    "# HELP graphfinder_server_up Whether the API process is running.\n# TYPE graphfinder_server_up gauge\ngraphfinder_server_up 1\n"
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
