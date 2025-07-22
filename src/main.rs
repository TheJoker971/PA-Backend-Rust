// src/main.rs

use axum::{
    Router, 
    routing::{get, post}, 
    Server,
};
use dotenvy::dotenv;
use std::{env, net::SocketAddr};
use tower_http::trace::TraceLayer;
use sqlx::PgPool;

mod db;
mod routes;
mod models;
mod auth;

#[tokio::main]
async fn main() {
    // Initialiser le système de logging
    tracing_subscriber::fmt::init();

    // Charger les variables d'environnement
    dotenv().ok();

    // Connexion à la base de données
    let pool: PgPool = db::init_db().await;

    println!("✅ Connexion à la base de données établie");

    // Routes simples pour commencer
    let app = Router::new()
        // Auth
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        // Health check
        .route("/health", get(routes::health_check))
        // Routes basiques
        .route("/properties", get(routes::get_properties))
        .route("/users", post(routes::create_user))
        // Layers
        .layer(TraceLayer::new_for_http())
        .with_state(pool.clone());

    // Détermination de l'adresse d'écoute
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");

    println!("🚀 Server running on http://{}", addr);

    // Démarrer le serveur
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}
