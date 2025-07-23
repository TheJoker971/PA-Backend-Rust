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

use crate::models::{CreateUserRequest, UpdateUserRoleRequest, Property, CreatePropertyRequest, UpdatePropertyStatusRequest, PropertyStatus, Investment, CreateInvestmentRequest, UpdateInvestmentRequest, User, UserRole};
use crate::auth::BearerAuthUser;

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
    let role_str = payload.role.unwrap_or_else(|| "user".to_string());
    let role: UserRole = role_str.into();
    
    match sqlx::query!(
        r#"INSERT INTO users (wallet, name, role)
        VALUES ($1, $2, $3)
        RETURNING id"#,
        payload.wallet,
        payload.name,
        role as UserRole
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

// Route publique pour lister uniquement les propriétés validées
pub async fn get_properties(
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"SELECT id, onchain_id, name, location, type, description, 
           total_price, token_price, annual_yield, image_url, documents, 
           created_at
           FROM properties 
           WHERE status = 'validated' 
           ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(rows) => {
            let properties: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "onchain_id": row.onchain_id,
                    "name": row.name,
                    "location": row.location,
                    "type": row.r#type,
                    "description": row.description,
                    "total_price": row.total_price,
                    "token_price": row.token_price,
                    "annual_yield": row.annual_yield,
                    "image_url": row.image_url,
                    "documents": row.documents,
                    "created_at": row.created_at
                })
            }).collect();
            
            (StatusCode::OK, Json(serde_json::json!({
                "properties": properties,
                "count": properties.len(),
                "message": "Propriétés validées uniquement"
            }))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour créer une property (manager ou admin requis)
pub async fn create_property(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePropertyRequest>,
) -> impl IntoResponse {
    // Vérifier le rôle
    if !matches!(user.role, UserRole::Admin | UserRole::Manager) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Accès manager ou admin requis"
        }))).into_response();
    }

    // Conversion des documents si nécessaire
    let documents = payload.documents.map(|d| {
        match d {
            serde_json::Value::Array(arr) => {
                arr.into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            },
            _ => vec![]
        }
    });

    match sqlx::query_as!(
        Property,
        r#"INSERT INTO properties (onchain_id, name, location, type, description, 
           total_price, token_price, annual_yield, image_url, documents, created_by, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending')
           RETURNING id, onchain_id, name, location, type as property_type, description, 
           total_price, token_price, annual_yield, image_url, documents, 
           created_by, created_at, status as "status: PropertyStatus", 
           status_updated_at, status_updated_by"#,
        payload.onchain_id,
        payload.name,
        payload.location,
        payload.property_type,
        payload.description,
        payload.total_price,
        payload.token_price,
        payload.annual_yield,
        payload.image_url,
        documents.as_deref(),
        user.id
    )
    .fetch_one(&pool)
    .await {
        Ok(property) => (StatusCode::CREATED, Json(serde_json::json!({
            "property": property,
            "message": "Propriété créée avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la création: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour récupérer toutes les properties (authentification requise)
/// Le comportement diffère selon le rôle de l'utilisateur :
/// - Admin: voit toutes les propriétés
/// - Manager: voit uniquement les propriétés qu'il a créées
/// - User: voit uniquement les propriétés dans lesquelles il a investi
pub async fn get_all_properties(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let properties_result = match user.role {
        UserRole::Admin => {
            sqlx::query_as!(
                Property,
                r#"SELECT id, onchain_id, name, location, type as property_type, description, 
                   total_price, token_price, annual_yield, image_url, documents, 
                   created_by, created_at, status as "status: PropertyStatus", 
                   status_updated_at, status_updated_by
                   FROM properties 
                   ORDER BY created_at DESC"#
            )
            .fetch_all(&pool)
            .await
        }
        UserRole::Manager => {
            sqlx::query_as!(
                Property,
                r#"SELECT id, onchain_id, name, location, type as property_type, description, 
                   total_price, token_price, annual_yield, image_url, documents, 
                   created_by, created_at, status as "status: PropertyStatus", 
                   status_updated_at, status_updated_by
                   FROM properties 
                   WHERE created_by = $1
                   ORDER BY created_at DESC"#,
                user.id
            )
            .fetch_all(&pool)
            .await
        }
        UserRole::User => {
            sqlx::query_as!(
                Property,
                r#"SELECT DISTINCT p.id, p.onchain_id, p.name, p.location, p.type as property_type, p.description, 
                   p.total_price, p.token_price, p.annual_yield, p.image_url, p.documents, 
                   p.created_by, p.created_at, p.status as "status: PropertyStatus", 
                   p.status_updated_at, p.status_updated_by
                   FROM properties p
                   JOIN investments i ON p.id = i.property_id
                   WHERE i.user_id = $1
                   ORDER BY p.created_at DESC"#,
                user.id
            )
            .fetch_all(&pool)
            .await
        }
    };

    match properties_result {
        Ok(properties) => (StatusCode::OK, Json(serde_json::json!({
            "properties": properties,
            "count": properties.len()
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour récupérer une property par ID (authentification requise)
pub async fn get_property_by_id(
    BearerAuthUser(_user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as!(
        Property,
        r#"SELECT id, onchain_id, name, location, type as property_type, description, 
           total_price, token_price, annual_yield, image_url, documents, 
           created_by, created_at, status as "status: PropertyStatus", 
           status_updated_at, status_updated_by
           FROM properties 
           WHERE id = $1"#,
        property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(property)) => (StatusCode::OK, Json(property)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Propriété non trouvée"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour mettre à jour une property (seulement si non validée)
pub async fn update_property(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<CreatePropertyRequest>,
) -> impl IntoResponse {
    // Vérifier le rôle
    if !matches!(user.role, UserRole::Admin | UserRole::Manager) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Accès manager ou admin requis"
        }))).into_response();
    }

    // Vérifier d'abord que la property existe et n'est pas validée
    let existing_property = match sqlx::query!(
        r#"SELECT status as "status: PropertyStatus" FROM properties WHERE id = $1"#,
        property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(prop)) => prop,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Propriété non trouvée"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Empêcher la modification si la property est validée (sauf pour l'admin)
    if matches!(existing_property.status, PropertyStatus::Validated) && !matches!(user.role, UserRole::Admin) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Impossible de modifier une propriété validée par l'admin"
        }))).into_response();
    }

    // Conversion des documents si nécessaire
    let documents = payload.documents.map(|d| {
        match d {
            serde_json::Value::Array(arr) => {
                arr.into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            },
            _ => vec![]
        }
    });

    match sqlx::query_as!(
        Property,
        r#"UPDATE properties SET 
           onchain_id = $2, name = $3, location = $4, type = $5, 
           description = $6, total_price = $7, token_price = $8, 
           annual_yield = $9, image_url = $10, documents = $11
           WHERE id = $1
           RETURNING id, onchain_id, name, location, type as property_type, description, 
           total_price, token_price, annual_yield, image_url, documents, 
           created_by, created_at, status as "status: PropertyStatus", 
           status_updated_at, status_updated_by"#,
        property_id,
        payload.onchain_id,
        payload.name,
        payload.location,
        payload.property_type,
        payload.description,
        payload.total_price,
        payload.token_price,
        payload.annual_yield,
        payload.image_url,
        documents.as_deref()
    )
    .fetch_one(&pool)
    .await {
        Ok(property) => (StatusCode::OK, Json(serde_json::json!({
            "property": property,
            "message": "Propriété mise à jour avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la mise à jour: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour mettre à jour le statut d'une property (admin seulement)
pub async fn update_property_status(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
    Json(payload): Json<UpdatePropertyStatusRequest>,
) -> impl IntoResponse {
    // Seul l'admin peut modifier le statut
    if !matches!(user.role, UserRole::Admin) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin peut modifier le statut des propriétés"
        }))).into_response();
    }

    // Vérifier que la property existe
    let property_exists = sqlx::query!(
        "SELECT id FROM properties WHERE id = $1",
        property_id
    )
    .fetch_optional(&pool)
    .await;

    match property_exists {
        Ok(Some(_)) => {},
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Propriété non trouvée"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    }

    match sqlx::query_as!(
        Property,
        r#"UPDATE properties SET 
           status = $2, status_updated_at = $3, status_updated_by = $4
           WHERE id = $1
           RETURNING id, onchain_id, name, location, type as property_type, description, 
           total_price, token_price, annual_yield, image_url, documents, 
           created_by, created_at, status as "status: PropertyStatus", 
           status_updated_at, status_updated_by"#,
        property_id,
        payload.status as PropertyStatus,
        Utc::now(),
        user.id
    )
    .fetch_one(&pool)
    .await {
        Ok(property) => (StatusCode::OK, Json(serde_json::json!({
            "property": property,
            "message": "Statut de la propriété mis à jour avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la mise à jour du statut: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour supprimer une property (admin seulement, et seulement si non validée)
pub async fn delete_property(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(property_id): Path<Uuid>,
) -> impl IntoResponse {
    // Seul l'admin peut supprimer
    if !matches!(user.role, UserRole::Admin) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin peut supprimer des propriétés"
        }))).into_response();
    }

    // Vérifier que la property existe et récupérer son statut
    let existing_property = match sqlx::query!(
        r#"SELECT status as "status: PropertyStatus" FROM properties WHERE id = $1"#,
        property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(prop)) => prop,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Propriété non trouvée"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Empêcher la suppression si la property est validée
    if matches!(existing_property.status, PropertyStatus::Validated) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Impossible de supprimer une propriété validée"
        }))).into_response();
    }

    match sqlx::query!("DELETE FROM properties WHERE id = $1", property_id)
        .execute(&pool)
        .await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "message": "Propriété supprimée avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la suppression: {}", e.to_string())
        }))).into_response(),
    }
}

// Routes pour les Investissements

/// Route pour récupérer tous les investissements (authentification requise)
pub async fn get_all_investments(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let investments_result = match user.role {
        UserRole::Admin => {
            sqlx::query_as!(
                Investment,
                r#"SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at
                   FROM investments 
                   ORDER BY created_at DESC"#
            )
            .fetch_all(&pool)
            .await
        }
        UserRole::Manager => {
            sqlx::query_as!(
                Investment,
                r#"SELECT i.id, i.user_id, i.property_id, i.amount_eth, i.shares, i.tx_hash, i.created_at
                   FROM investments i
                   JOIN properties p ON i.property_id = p.id
                   WHERE p.created_by = $1
                   ORDER BY i.created_at DESC"#,
                user.id
            )
            .fetch_all(&pool)
            .await
        }
        UserRole::User => {
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
        }
    };

    match investments_result {
        Ok(investments) => (StatusCode::OK, Json(serde_json::json!({
            "investments": investments,
            "count": investments.len()
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour créer un investissement (tous les utilisateurs authentifiés)
pub async fn create_investment(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateInvestmentRequest>,
) -> impl IntoResponse {
    // Vérifier que la propriété existe et est validée
    let property_status = match sqlx::query!(
        r#"SELECT status as "status: PropertyStatus" FROM properties WHERE id = $1"#,
        payload.property_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(prop)) => prop.status,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Propriété non trouvée"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Seules les propriétés validées peuvent recevoir des investissements
    if !matches!(property_status, PropertyStatus::Validated) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Impossible d'investir dans une propriété non validée"
        }))).into_response();
    }

    match sqlx::query_as!(
        Investment,
        r#"INSERT INTO investments (user_id, property_id, amount_eth, shares, tx_hash)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, user_id, property_id, amount_eth, shares, tx_hash, created_at"#,
        user.id,
        payload.property_id,
        payload.amount_eth,
        payload.shares,
        payload.tx_hash
    )
    .fetch_one(&pool)
    .await {
        Ok(investment) => (StatusCode::CREATED, Json(serde_json::json!({
            "investment": investment,
            "message": "Investissement créé avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la création: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour récupérer un investissement par ID
pub async fn get_investment_by_id(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(investment_id): Path<Uuid>,
) -> impl IntoResponse {
    let investment = match sqlx::query_as!(
        Investment,
        r#"SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at
           FROM investments 
           WHERE id = $1"#,
        investment_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(inv)) => inv,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Investissement non trouvé"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    };

    // Contrôle d'accès selon le rôle
    let has_access = match user.role {
        UserRole::Admin => true,
        UserRole::User => investment.user_id == user.id,
        UserRole::Manager => {
            // Vérifier si la propriété appartient au manager
            match sqlx::query!(
                "SELECT created_by FROM properties WHERE id = $1",
                investment.property_id
            )
            .fetch_optional(&pool)
            .await {
                Ok(Some(prop)) => prop.created_by == user.id,
                _ => false,
            }
        }
    };

    if !has_access {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Accès non autorisé à cet investissement"
        }))).into_response();
    }

    (StatusCode::OK, Json(investment)).into_response()
}

/// Route pour mettre à jour un investissement
pub async fn update_investment(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(investment_id): Path<Uuid>,
    Json(payload): Json<UpdateInvestmentRequest>,
) -> impl IntoResponse {
    // Vérifier que l'investissement existe et récupérer ses infos
    let existing_investment = match sqlx::query!(
        "SELECT user_id FROM investments WHERE id = $1",
        investment_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(inv)) => inv,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Investissement non trouvé"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Contrôle d'accès : seul l'admin ou le propriétaire peut modifier
    if !matches!(user.role, UserRole::Admin) && existing_investment.user_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin ou le propriétaire peut modifier cet investissement"
        }))).into_response();
    }

    match sqlx::query_as!(
        Investment,
        r#"UPDATE investments SET 
           amount_eth = $2, shares = $3, tx_hash = $4
           WHERE id = $1
           RETURNING id, user_id, property_id, amount_eth, shares, tx_hash, created_at"#,
        investment_id,
        payload.amount_eth,
        payload.shares,
        payload.tx_hash
    )
    .fetch_one(&pool)
    .await {
        Ok(investment) => (StatusCode::OK, Json(serde_json::json!({
            "investment": investment,
            "message": "Investissement mis à jour avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la mise à jour: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour supprimer un investissement
pub async fn delete_investment(
    BearerAuthUser(user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(investment_id): Path<Uuid>,
) -> impl IntoResponse {
    // Vérifier que l'investissement existe et récupérer ses infos
    let existing_investment = match sqlx::query!(
        "SELECT user_id FROM investments WHERE id = $1",
        investment_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(inv)) => inv,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Investissement non trouvé"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Contrôle d'accès : seul l'admin ou le propriétaire peut supprimer
    if !matches!(user.role, UserRole::Admin) && existing_investment.user_id != user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin ou le propriétaire peut supprimer cet investissement"
        }))).into_response();
    }

    match sqlx::query!("DELETE FROM investments WHERE id = $1", investment_id)
        .execute(&pool)
        .await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "message": "Investissement supprimé avec succès"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la suppression: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour mettre à jour le rôle d'un utilisateur (admin seulement)
pub async fn update_user_role(
    BearerAuthUser(admin_user): BearerAuthUser,
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    // Seul l'admin peut modifier les rôles
    if !matches!(admin_user.role, UserRole::Admin) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin peut modifier les rôles des utilisateurs"
        }))).into_response();
    }

    // Convertir le rôle string en enum
    let new_role: UserRole = payload.role.into();
    let role_display = new_role; // Copy pour le message

    // Vérifier que l'utilisateur existe
    let existing_user = match sqlx::query!(
        r#"SELECT id, wallet, name, role as "role: UserRole" FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(user)) => user,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Utilisateur non trouvé"
        }))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la vérification: {}", e.to_string())
        }))).into_response(),
    };

    // Empêcher l'admin de modifier son propre rôle
    if existing_user.id == admin_user.id {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Impossible de modifier son propre rôle"
        }))).into_response();
    }

    // Mettre à jour le rôle
    match sqlx::query_as!(
        User,
        r#"UPDATE users SET role = $2
           WHERE id = $1
           RETURNING id, wallet, name, role as "role: UserRole", created_at"#,
        user_id,
        new_role as UserRole
    )
    .fetch_one(&pool)
    .await {
        Ok(updated_user) => (StatusCode::OK, Json(serde_json::json!({
            "user": updated_user,
            "message": format!("Rôle de l'utilisateur mis à jour vers '{}'", role_display)
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la mise à jour: {}", e.to_string())
        }))).into_response(),
    }
}

/// Route pour lister tous les utilisateurs (admin seulement)
pub async fn get_all_users(
    BearerAuthUser(admin_user): BearerAuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Seul l'admin peut voir tous les utilisateurs
    if !matches!(admin_user.role, UserRole::Admin) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": "Seul l'admin peut voir tous les utilisateurs"
        }))).into_response();
    }

    match sqlx::query_as!(
        User,
        r#"SELECT id, wallet, name, role as "role: UserRole", created_at
           FROM users 
           ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(users) => (StatusCode::OK, Json(serde_json::json!({
            "users": users,
            "count": users.len()
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": format!("Erreur lors de la récupération: {}", e.to_string())
        }))).into_response(),
    }
}