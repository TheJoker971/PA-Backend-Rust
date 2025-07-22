// routes.rs

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;

use crate::models::{CreateUserRequest};

// Route de santé
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "message": "API is running"
    }))
}

// Route simple pour créer un utilisateur
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let role = payload.role.unwrap_or_else(|| "user".to_string());
    
    match sqlx::query!(
        r#"INSERT INTO users (signature, name, role)
        VALUES ($1, $2, $3)
        RETURNING id"#,
        payload.signature,
        payload.name,
        role
    )
    .fetch_one(&pool)
    .await {
        Ok(record) => (StatusCode::CREATED, Json(serde_json::json!({ 
            "id": record.id,
            "message": "Utilisateur créé avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
            "error": format!("Erreur lors de la création: {}", e.to_string())
        }))).into_response(),
    }
}

// Route simple pour lister les propriétés (version basique)
pub async fn get_properties(
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"SELECT id, name, location FROM properties WHERE is_validated = true ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(rows) => {
            let properties: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "name": row.name,
                    "location": row.location
                })
            }).collect();
            
            (StatusCode::OK, Json(serde_json::json!({
                "properties": properties,
                "count": properties.len()
            }))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}