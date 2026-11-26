use axum::extract::Request;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .layer(middleware::from_fn(request_id))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
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
