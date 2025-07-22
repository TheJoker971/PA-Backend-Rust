/// src/auth.rs
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{Utc, Duration};
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

/// Handler `POST /auth/login`
pub async fn login(
    jar: CookieJar,
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    // Récupérer l'utilisateur par signature
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
        _ => return (StatusCode::UNAUTHORIZED, "Signature invalide").into_response(),
    };

    // Générer un token de session
    let token = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(24);

    // Insérer en base
    sqlx::query!(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3)",
        token,
        user.id,
        expires_at
    )
    .execute(&pool)
    .await
    .unwrap();

    // Créer le cookie sécurisé
    let cookie = Cookie::build("session_token", token.to_string())
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .path("/")
        .finish();

    let jar = jar.add(cookie);

    let session_user = SessionUser {
        id: user.id,
        signature: user.signature,
        name: user.name,
        role: user.role,
        created_at: user.created_at,
    };

    (jar, (StatusCode::OK, Json(session_user))).into_response()
}

/// Handler `POST /auth/logout`
pub async fn logout(
    jar: CookieJar,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session_token") {
        let token = Uuid::parse_str(cookie.value()).unwrap();
        let _ = sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&pool)
            .await;
    }
    // Supprimer le cookie côté client
    let jar = jar.remove(Cookie::named("session_token"));
    (jar, StatusCode::OK)
}

/// Extracteur d'utilisateur authentifié
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
        let token = Uuid::parse_str(cookie.value()).map_err(|_| (StatusCode::UNAUTHORIZED, "Token invalide"))?;

        // Récupérer le pool
        let pool = parts.extensions
            .get::<PgPool>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Pool manquant"))?
            .clone();

        // Vérifier la session
        let user = sqlx::query_as!(
            User,
            r#"SELECT u.id, u.signature, u.name, u.role, u.created_at
               FROM sessions s JOIN users u ON u.id = s.user_id
               WHERE s.token = $1 AND s.expires_at > NOW()"#, token
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Session invalide"))?;

        if let Some(u) = user {
            Ok(AuthUser(SessionUser {
                id: u.id,
                signature: u.signature,
                name: u.name,
                role: u.role,
                created_at: u.created_at,
            }))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Session expirée"))
        }
    }
}

/// Middleware simple qui vérifie le rôle admin
pub async fn require_admin_role(
    AuthUser(user): AuthUser,
) -> Result<AuthUser, Response> {
    if user.role == "admin" {
        Ok(AuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès admin requis").into_response())
    }
}

/// Middleware simple qui vérifie le rôle manager ou admin
pub async fn require_manager_or_admin_role(
    AuthUser(user): AuthUser,
) -> Result<AuthUser, Response> {
    if user.role == "admin" || user.role == "manager" {
        Ok(AuthUser(user))
    } else {
        Err((StatusCode::FORBIDDEN, "Accès manager ou admin requis").into_response())
    }
}