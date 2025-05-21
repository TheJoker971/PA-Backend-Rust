use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{User, Property, Investment};

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

async fn get_users(State(pool): State<PgPool>) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(users)
}

// === Properties ===
#[derive(Deserialize)]
struct NewProperty {
    onchain_id: i32,
    name: String,
    description: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
    created_by: Option<Uuid>,
}

async fn create_property(
    State(pool): State<PgPool>,
    Json(payload): Json<NewProperty>,
) -> Json<String> {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO properties (id, onchain_id, name, description, image_url, category, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        id,
        payload.onchain_id,
        payload.name,
        payload.description,
        payload.image_url,
        payload.category,
        payload.created_by
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(format!("Property {} created", id))
}

async fn get_properties(State(pool): State<PgPool>) -> Json<Vec<Property>> {
    let properties = sqlx::query_as::<_, Property>("SELECT * FROM properties")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(properties)
}

// === Investments ===
#[derive(Deserialize)]
struct NewInvestment {
    user_id: Uuid,
    property_id: Uuid,
    amount_eth: f64,
    shares: i32,
    tx_hash: String,
}

async fn create_investment(
    State(pool): State<PgPool>,
    Json(payload): Json<NewInvestment>,
) -> Json<String> {
    let id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO investments (id, user_id, property_id, amount_eth, shares, tx_hash)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        id,
        payload.user_id,
        payload.property_id,
        payload.amount_eth,
        payload.shares,
        payload.tx_hash
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(format!("Investment {} created", id))
}

async fn get_investments(State(pool): State<PgPool>) -> Json<Vec<Investment>> {
    let investments = sqlx::query_as::<_, Investment>("SELECT * FROM investments")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(investments)
}

// === Router ===
pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users", get(get_users))
        .route("/properties", post(create_property))
        .route("/properties", get(get_properties))
        .route("/investments", post(create_investment))
        .route("/investments", get(get_investments))
        .with_state(pool)
}
