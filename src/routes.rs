// routes.rs

use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::auth::AuthUser;
use crate::models::{Property, Investment, CreatePropertyRequest, CreateInvestmentRequest, ValidatePropertyRequest};

// Route de santé
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

// Routes pour les propriétés
pub async fn create_property(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePropertyRequest>,
) -> impl IntoResponse {
    // Vérification du rôle déjà faite par le middleware
    
    // Création de la propriété
    let result = sqlx::query!(
        r#"INSERT INTO properties 
        (onchain_id, name, location, type, description, total_price, token_price, annual_yield, image_url, documents, created_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id"#,
        payload.onchain_id,
        payload.name,
        payload.location,
        payload.property_type,
        payload.description,
        payload.total_price,
        payload.token_price,
        payload.annual_yield,
        payload.image_url,
        payload.documents,
        user.id
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(record) => (StatusCode::CREATED, Json(serde_json::json!({ "id": record.id }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_properties(
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Récupérer toutes les propriétés validées
    match sqlx::query_as!(
        Property,
        r#"SELECT 
            id, 
            onchain_id, 
            name, 
            location, 
            type as "property_type: String", 
            description, 
            total_price, 
            token_price, 
            annual_yield, 
            image_url, 
            documents, 
            created_by, 
            created_at,
            is_validated,
            validated_at,
            validated_by
        FROM properties 
        WHERE is_validated = true
        ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(properties) => (StatusCode::OK, Json(properties)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_all_properties(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Vérification du rôle déjà faite par le middleware
    
    // Récupérer toutes les propriétés (validées et non validées)
    match sqlx::query_as!(
        Property,
        r#"SELECT 
            id, 
            onchain_id, 
            name, 
            location, 
            type as "property_type: String", 
            description, 
            total_price, 
            token_price, 
            annual_yield, 
            image_url, 
            documents, 
            created_by, 
            created_at,
            is_validated,
            validated_at,
            validated_by
        FROM properties 
        ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(properties) => (StatusCode::OK, Json(properties)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_property(
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
) -> impl IntoResponse {
    // Récupérer une propriété par son ID (uniquement si validée)
    match sqlx::query_as!(
        Property,
        r#"SELECT 
            id, 
            onchain_id, 
            name, 
            location, 
            type as "property_type: String", 
            description, 
            total_price, 
            token_price, 
            annual_yield, 
            image_url, 
            documents, 
            created_by, 
            created_at,
            is_validated,
            validated_at,
            validated_by
        FROM properties 
        WHERE id = $1 AND is_validated = true"#,
        property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(property)) => (StatusCode::OK, Json(property)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Propriété non trouvée" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_property_admin(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
) -> impl IntoResponse {
    // Récupérer une propriété par son ID (validée ou non)
    match sqlx::query_as!(
        Property,
        r#"SELECT 
            id, 
            onchain_id, 
            name, 
            location, 
            type as "property_type: String", 
            description, 
            total_price, 
            token_price, 
            annual_yield, 
            image_url, 
            documents, 
            created_by, 
            created_at,
            is_validated,
            validated_at,
            validated_by
        FROM properties 
        WHERE id = $1"#,
        property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(property)) => (StatusCode::OK, Json(property)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Propriété non trouvée" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn validate_property(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<ValidatePropertyRequest>,
) -> impl IntoResponse {
    // Vérification du rôle déjà faite par le middleware (admin uniquement)
    
    // Mettre à jour le statut de validation de la propriété
    let now = Utc::now().naive_utc();
    let result = sqlx::query!(
        r#"UPDATE properties 
        SET is_validated = $1, validated_at = $2, validated_by = $3
        WHERE id = $4
        RETURNING id"#,
        payload.is_validated,
        now,
        user.id,
        property_id
    )
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(_)) => (StatusCode::OK, Json(serde_json::json!({ 
            "message": if payload.is_validated { "Propriété validée avec succès" } else { "Propriété invalidée avec succès" }
        }))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Propriété non trouvée" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

// Routes pour les investissements
pub async fn create_investment(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateInvestmentRequest>,
) -> impl IntoResponse {
    // Vérifier que la propriété est validée
    let property = sqlx::query!(
        r#"SELECT is_validated FROM properties WHERE id = $1"#,
        payload.property_id
    )
    .fetch_optional(&pool)
    .await;

    match property {
        Ok(Some(prop)) if prop.is_validated => {
            // La propriété existe et est validée, on peut créer l'investissement
            let result = sqlx::query!(
                r#"INSERT INTO investments 
                (user_id, property_id, amount_eth, shares, tx_hash)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id"#,
                user.id,
                payload.property_id,
                payload.amount_eth,
                payload.shares,
                payload.tx_hash
            )
            .fetch_one(&pool)
            .await;

            match result {
                Ok(record) => (StatusCode::CREATED, Json(serde_json::json!({ "id": record.id }))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
            }
        },
        Ok(Some(_)) => (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Cette propriété n'est pas encore validée" }))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Propriété non trouvée" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_investments(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Les admins et managers peuvent voir tous les investissements
    let investments = if user.role == "admin" || user.role == "manager" {
        sqlx::query_as!(
            Investment,
            r#"SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at 
            FROM investments 
            ORDER BY created_at DESC"#
        )
        .fetch_all(&pool)
        .await
    } else {
        // Les utilisateurs normaux ne voient que leurs propres investissements
        sqlx::query_as!(
            Investment,
            r#"SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at 
            FROM investments 
            WHERE user_id = $1
            ORDER BY created_at DESC"#,
            user.id
        )
        .fetch_all(&pool)
        .await
    };

    match investments {
        Ok(investments) => (StatusCode::OK, Json(investments)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn get_user_investments(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    // Vérifier que l'utilisateur est admin/manager ou qu'il consulte ses propres investissements
    if user.role != "admin" && user.role != "manager" && user.id != user_id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Accès refusé" }))).into_response();
    }

    // Récupérer les investissements de l'utilisateur
    match sqlx::query_as!(
        Investment,
        r#"SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at 
        FROM investments 
        WHERE user_id = $1
        ORDER BY created_at DESC"#,
        user_id
    )
    .fetch_all(&pool)
    .await {
        Ok(investments) => (StatusCode::OK, Json(investments)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

// Route pour créer un utilisateur
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<crate::models::CreateUserRequest>,
) -> impl IntoResponse {
    // Créer l'utilisateur
    let result = sqlx::query!(
        r#"INSERT INTO users (signature, name)
        VALUES ($1, $2)
        RETURNING id"#,
        payload.signature,
        payload.name
    )
    .fetch_one(&pool)
    .await;

    match result {
        Ok(record) => (StatusCode::CREATED, Json(serde_json::json!({ "id": record.id }))),
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                (StatusCode::CONFLICT, Json(serde_json::json!({ "error": "Utilisateur déjà existant" })))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })))
            }
        }
    }
}

// Route pour récupérer les utilisateurs
pub async fn get_users(
    AuthUser(user): AuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Seuls les admins et managers peuvent voir tous les utilisateurs
    if user.role != "admin" && user.role != "manager" {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": "Accès refusé" }))).into_response();
    }

    // Récupérer tous les utilisateurs avec leurs rôles
    match sqlx::query!(
        r#"SELECT 
            u.id, 
            u.wallet, 
            u.email, 
            u.name, 
            u.created_at,
            COALESCE(r.role, 'user') as role
        FROM users u
        LEFT JOIN roles r ON u.wallet_short = r.wallet_short
        ORDER BY u.created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(rows) => {
            let users = rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "wallet": row.wallet,
                    "email": row.email,
                    "name": row.name,
                    "role": row.role,
                    "created_at": row.created_at
                })
            }).collect::<Vec<_>>();
            
            (StatusCode::OK, Json(users))
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}