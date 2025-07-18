/// src/auth.rs
use axum::{
    extract::{FromRequest, RequestParts, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use bcrypt::verify;
use crate::models::{User, SessionUser};

/// Structure renvoyée après connexion
#[derive(Debug, Serialize)]
pub struct SessionUser {
    pub id: Uuid,
    pub wallet: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: String,
    pub created_at: chrono::NaiveDateTime,
}

/// Payload JSON pour le login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Handler `POST /auth/login`
pub async fn login(
    jar: CookieJar,
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    // Récupérer l'utilisateur par email
    let row = sqlx::query!(
        r#"SELECT id, wallet, email, name, role, created_at, password_hash
           FROM users
           WHERE email = $1"#, payload.email
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    let user = match row {
        Some(u) if verify(&payload.password, &u.password_hash).unwrap_or(false) => u,
        _ => return (StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response(),
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
        wallet: user.wallet,
        email: user.email,
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

    async fn from_request(req: &mut RequestParts<S>) -> Result<Self, Self::Rejection> {
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
        let row = sqlx::query!(
            r#"SELECT u.id, u.wallet, u.email, u.name, u.role, u.created_at
               FROM sessions s JOIN users u ON u.id = s.user_id
               WHERE s.token = $1 AND s.expires_at > NOW()"#, token
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Session invalide"))?;

        if let Some(u) = row {
            Ok(AuthUser(SessionUser {
                id: u.id,
                wallet: u.wallet,
                email: u.email,
                name: u.name,
                role: u.role,
                created_at: u.created_at,
            }))
        } else {
            Err((StatusCode::UNAUTHORIZED, "Session expirée"))
        }
    }
}


// ------------------------------------------
// src/models.rs (ajouts)

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub token: Uuid,
    pub user_id: Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Ajoute au struct User dans models.rs :
// #[serde(skip_serializing)]
// pub password_hash: String,


// ------------------------------------------
// src/main.rs

use axum::{
    Router, 
    routing::{get, post, delete, put}, 
    response::IntoResponse,
    middleware::{self, from_fn_with_state},
    extract::State,
};
use dotenvy::dotenv;
use std::{env, fs};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use axum_extra::extract::cookie::CookieJarLayer;
use tower_csrf::{CsurfLayer, CsrfConfig, SecretGenerator};
use sqlx::PgPool;

mod db;
mod routes;
mod models;
mod auth;

async fn require_admin_role<B>(
    auth_user: auth::AuthUser,
    request: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, (axum::http::StatusCode, &'static str)> {
    if auth_user.0.role != "admin" {
        return Err((axum::http::StatusCode::FORBIDDEN, "Accès réservé aux administrateurs"));
    }
    Ok(next.run(request).await)
}

async fn require_manager_or_admin_role<B>(
    auth_user: auth::AuthUser,
    request: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, (axum::http::StatusCode, &'static str)> {
    if auth_user.0.role != "admin" && auth_user.0.role != "manager" {
        return Err((axum::http::StatusCode::FORBIDDEN, "Accès réservé aux administrateurs et managers"));
    }
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    // Charger les variables d'environnement
    dotenv().ok();

    // Connexion à la base de données
    let pool: PgPool = db::init_db().await;

    // Exécuter les migrations SQL (si nécessaire)
    let schema_sql = fs::read_to_string("migrations/schema.sql")
        .expect("Failed to read schema.sql");
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("Failed to execute: {}\nError: {}", trimmed, e));
        }
    }

    // Configurer CSRF
    let secret = SecretGenerator::new().generate();
    let csrf_layer = CsurfLayer::new(CsrfConfig::new(secret));

    // Routes pour les rôles (protégées par admin)
    let roles_routes = Router::new()
        .route("/", post(auth::create_role))
        .route("/", get(auth::get_roles))
        .route("/:role_id", delete(auth::delete_role))
        .route("/signature/:signature", get(auth::get_signature_role))
        .route_layer(middleware::from_fn(require_admin_role));

    // Routes pour les propriétés
    let properties_routes = Router::new()
        // Routes publiques (propriétés validées uniquement)
        .route("/", get(routes::get_properties))
        .route("/:property_id", get(routes::get_property))
        // Routes protégées pour admin/manager
        .route("/all", get(routes::get_all_properties)
            .route_layer(middleware::from_fn(require_manager_or_admin_role)))
        .route("/admin/:property_id", get(routes::get_property_admin)
            .route_layer(middleware::from_fn(require_manager_or_admin_role)))
        .route("/", post(routes::create_property)
            .route_layer(middleware::from_fn(require_manager_or_admin_role)))
        // Route de validation (admin uniquement)
        .route("/:property_id/validate", put(routes::validate_property)
            .route_layer(middleware::from_fn(require_admin_role)));

    // Routes pour les investissements
    let investments_routes = Router::new()
        .route("/", post(routes::create_investment))
        .route("/", get(routes::get_investments))
        .route("/user/:user_id", get(routes::get_user_investments));

    // Routes pour les utilisateurs
    let users_routes = Router::new()
        .route("/", post(routes::create_user))
        .route("/", 
            get(routes::get_users)
                .route_layer(middleware::from_fn(require_manager_or_admin_role))
        );

    // Construire le router principal
    let app = Router::new()
        // Auth
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        // Health check
        .route("/health", get(routes::health_check))
        // Nested routes
        .nest("/roles", roles_routes)
        .nest("/properties", properties_routes)
        .nest("/investments", investments_routes)
        .nest("/users", users_routes)
        // Layers
        .layer(CookieJarLayer::new())
        .layer(csrf_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(pool.clone());

    // Détermination de l'adresse d'écoute
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Création du listener TCP
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("🚀 Server running on http://{}", addr);

    // Démarrer le serveur
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
