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
