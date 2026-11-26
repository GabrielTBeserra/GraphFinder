use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/tokens", post(create_token))
        .route("/api/v1/tokens/{id}", axum::routing::delete(revoke_token))
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
        }
    }
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
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "source lookup failed"))?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(axum::Json(sources))
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
    Ok(axum::Json(source.into()))
}

async fn get_source(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::Json<SourceResponse>, (StatusCode, &'static str)> {
    let principal = require_principal(&pool, &headers).await?;
    let source =
        authorize_source_or_error(&pool, &principal, id, gf_storage::SCOPE_SEARCH_READ).await?;
    Ok(axum::Json(source.into()))
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
    Ok(axum::Json(updated.into()))
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
