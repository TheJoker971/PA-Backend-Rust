use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// === Health Check ===
#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

// === Users ===
#[derive(Deserialize)]
struct NewUser {
    wallet: String,
    email: Option<String>,
    name: Option<String>,
    role: Option<String>,
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<NewUser>,
) -> Json<String> {
    let id = Uuid::new_v4();
    let role = payload.role.unwrap_or_else(|| "user".to_string());

    sqlx::query!(
        r#"
        INSERT INTO users (id, wallet, email, name, role)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        id,
        payload.wallet,
        payload.email,
        payload.name,
        role
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(format!("User {} created", id))
}

// === Router ===
pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .with_state(pool)
}
