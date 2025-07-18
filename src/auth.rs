/// src/auth.rs
use axum::{
    extract::{FromRequest, Request, State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{User, UserWithRole, Role};

/// Structure renvoyée après connexion
#[derive(Debug, Serialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub signature: String,
    pub name: Option<String>,
    pub role: String,
    pub created_at: chrono::NaiveDateTime,
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
) -> impl IntoResponse {
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
        .expires(expires_at.into())
        .finish();

    let jar = jar.add(cookie);

    let session_user = SessionUser {
        id: user.id,
        signature: user.signature,
        name: user.name,
        role: user.role,
        created_at: user.created_at,
    };

    (jar, (StatusCode::OK, Json(session_user)))
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
impl<S> FromRequest<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut Request, state: &S) -> Result<Self, Self::Rejection> {
        // Récupérer le cookie
        let jar = CookieJar::from_request(req)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Cookies manquants"))?;
        let cookie = jar.get("session_token").ok_or((StatusCode::UNAUTHORIZED, "Non authentifié"))?;
        let token = Uuid::parse_str(cookie.value()).map_err(|_| (StatusCode::UNAUTHORIZED, "Token invalide"))?;

        // Récupérer le pool
        let pool = req.extensions()
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

/// Middleware pour vérifier les rôles
pub async fn require_role(
    auth_user: AuthUser,
    allowed_roles: Vec<&'static str>,
) -> Result<AuthUser, (StatusCode, &'static str)> {
    if allowed_roles.contains(&auth_user.0.role.as_str()) {
        Ok(auth_user)
    } else {
        Err((StatusCode::FORBIDDEN, "Accès refusé"))
    }
}

/// Handler pour créer un rôle
pub async fn create_role(
    auth_user: AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<crate::models::CreateRoleRequest>,
) -> impl IntoResponse {
    // Vérifier que l'utilisateur est admin
    if auth_user.0.role != "admin" {
        return (StatusCode::FORBIDDEN, "Accès refusé").into_response();
    }

    // Insérer le rôle avec la signature
    match sqlx::query!(
        r#"INSERT INTO roles (signature, role) VALUES ($1, $2)
           ON CONFLICT (signature) DO UPDATE SET role = $2
           RETURNING id"#,
        payload.signature,
        payload.role
    )
    .fetch_one(&pool)
    .await {
        Ok(record) => (StatusCode::CREATED, Json(serde_json::json!({ "id": record.id }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// Handler pour récupérer tous les rôles
pub async fn get_roles(
    auth_user: AuthUser,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    // Vérifier que l'utilisateur est admin
    if auth_user.0.role != "admin" {
        return (StatusCode::FORBIDDEN, "Accès refusé").into_response();
    }

    // Récupérer tous les rôles
    match sqlx::query_as!(
        Role,
        r#"SELECT id, wallet_short, role, created_at, updated_at FROM roles ORDER BY updated_at DESC"#
    )
    .fetch_all(&pool)
    .await {
        Ok(roles) => (StatusCode::OK, Json(roles)),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

/// Handler pour supprimer un rôle
pub async fn delete_role(
    auth_user: AuthUser,
    State(pool): State<PgPool>,
    Path(role_id): Path<Uuid>,
) -> impl IntoResponse {
    // Vérifier que l'utilisateur est admin
    if auth_user.0.role != "admin" {
        return (StatusCode::FORBIDDEN, "Accès refusé").into_response();
    }

    // Supprimer le rôle
    match sqlx::query!(
        r#"DELETE FROM roles WHERE id = $1"#,
        role_id
    )
    .execute(&pool)
    .await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

/// Handler pour récupérer le rôle d'un utilisateur par signature
pub async fn get_signature_role(
    State(pool): State<PgPool>,
    Path(signature): Path<String>,
) -> impl IntoResponse {
    // Récupérer le rôle
    match sqlx::query_scalar!(
        r#"SELECT role FROM roles WHERE signature = $1"#,
        signature
    )
    .fetch_optional(&pool)
    .await {
        Ok(Some(role)) => (StatusCode::OK, Json(serde_json::json!({ "role": role }))),
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({ "role": "user" }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))),
    }
}