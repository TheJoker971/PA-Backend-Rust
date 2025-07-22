/// src/auth.rs
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::User;

/// Structure renvoyée après connexion
#[derive(Debug, Serialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub signature: String,
    pub name: Option<String>,
    pub role: String,
    pub created_at: chrono::DateTime<Utc>,
}

/// Payload JSON pour le login par signature
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub signature: String,
}

/// Payload JSON pour l'authentification par bearer token
#[derive(Debug, Deserialize)]
pub struct BearerAuthRequest {
    pub signature: String,
}

/// Handler `POST /auth/login` (simplifié sans sessions)
pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    // Vérifier si l'utilisateur existe déjà
    let user = match sqlx::query_as!(
        User,
        r#"SELECT id, signature, name, role, created_at
           FROM users
           WHERE signature = $1"#, payload.signature
    )
    .fetch_optional(&pool)
    .await
    .unwrap() {
        Some(u) => u,
        None => {
            // Créer automatiquement l'utilisateur s'il n'existe pas
            match sqlx::query_as!(
                User,
                r#"INSERT INTO users (signature, name, role)
                   VALUES ($1, $2, $3)
                   RETURNING id, signature, name, role, created_at"#,
                payload.signature,
                "Utilisateur Wallet", // Nom par défaut
                "user" // Rôle par défaut
            )
            .fetch_one(&pool)
            .await {
                Ok(new_user) => new_user,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Erreur lors de la création de l'utilisateur").into_response(),
            }
        }
    };

    let session_user = SessionUser {
        id: user.id,
        signature: user.signature,
        name: user.name,
        role: user.role.unwrap_or_else(|| "user".to_string()),
        created_at: user.created_at,
    };

    (StatusCode::OK, Json(session_user)).into_response()
}

/// Handler `POST /auth/logout` (simplifié)
pub async fn logout() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"message": "Déconnecté avec succès"})))
}

/// Extracteur d'utilisateur authentifié (cookies - conservé pour compatibilité)
pub struct AuthUser(pub SessionUser);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Récupérer le cookie
        let jar = CookieJar::from_request_parts(parts, _state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Cookies manquants"))?;
        let cookie = jar.get("session_token").ok_or((StatusCode::UNAUTHORIZED, "Non authentifié"))?;
        let _token = Uuid::parse_str(cookie.value()).map_err(|_| (StatusCode::UNAUTHORIZED, "Token invalide"))?;

        // Pour la compatibilité, retourner une erreur car on n'utilise plus les sessions
        Err((StatusCode::UNAUTHORIZED, "Utiliser l'authentification Bearer Token"))
    }
}

/// Extracteur d'utilisateur authentifié via Bearer Token
pub struct BearerAuthUser(pub SessionUser);

#[axum::async_trait]
impl<S> FromRequestParts<S> for BearerAuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Récupérer le header Authorization
        let headers = &parts.headers;
        let auth_header = headers
            .get("Authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "Header Authorization manquant"))?
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Header Authorization invalide"))?;

        // Vérifier que c'est un Bearer token
        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Token Bearer requis"));
        }

        let signature = auth_header.strip_prefix("Bearer ").unwrap().trim();

        // Pour l'instant, on va utiliser une approche simplifiée
        // On va créer un utilisateur temporaire basé sur la signature
        // Dans un vrai système, on récupérerait l'utilisateur depuis la DB
        Ok(BearerAuthUser(SessionUser {
            id: Uuid::new_v4(), // ID temporaire
            signature: signature.to_string(),
            name: Some("Utilisateur Wallet".to_string()),
            role: "admin".to_string(), // Rôle par défaut pour les tests
            created_at: Utc::now(),
        }))
    }
}

/// Middleware qui vérifie le rôle admin avec Bearer Token
pub async fn require_admin_bearer(
    BearerAuthUser(user): BearerAuthUser,
) -> Result<BearerAuthUser, Response> {
    if user.role == "admin" {
        Ok(BearerAuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès admin requis").into_response())
    }
}

/// Middleware qui vérifie le rôle manager ou admin avec Bearer Token
pub async fn require_manager_or_admin_bearer(
    BearerAuthUser(user): BearerAuthUser,
) -> Result<BearerAuthUser, Response> {
    if user.role == "admin" || user.role == "manager" {
        Ok(BearerAuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès manager ou admin requis").into_response())
    }
}

/// Middleware simple qui vérifie le rôle admin (pour compatibilité)
pub async fn require_admin_role(
    AuthUser(user): AuthUser,
) -> Result<AuthUser, Response> {
    if user.role == "admin" {
        Ok(AuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès admin requis").into_response())
    }
}

/// Middleware simple qui vérifie le rôle manager ou admin (pour compatibilité)
pub async fn require_manager_or_admin_role(
    AuthUser(user): AuthUser,
) -> Result<AuthUser, Response> {
    if user.role == "admin" || user.role == "manager" {
        Ok(AuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès manager ou admin requis").into_response())
    }
}