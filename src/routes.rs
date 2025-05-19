use axum::{routing::get, Router, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub fn create_router() -> Router {
    Router::new().route("/health", get(health_check))
}
