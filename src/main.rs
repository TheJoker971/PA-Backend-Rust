// src/main.rs

use axum::{
    Router, 
    routing::{get, post, put, delete}, 
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

    // Configuration des routes avec authentification Bearer Token
    let app = Router::new()
        // Auth - routes de connexion/déconnexion (conservées pour compatibilité)
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        
        // Health check (publique)
        .route("/health", get(routes::health_check))
        
        // Routes utilisateurs
        .route("/users", post(routes::create_user))
        
        // Routes properties avec authentification Bearer Token
        // Routes publiques (anciennes pour compatibilité)
        .route("/properties/public", get(routes::get_properties))
        
        // Routes protégées par Bearer Token
        .route("/api/properties", 
            get(routes::get_all_properties)
            .post(routes::create_property)
        )
        .route("/api/properties/:id", 
            get(routes::get_property_by_id)
            .put(routes::update_property)
            .delete(routes::delete_property)
        )
        .route("/api/properties/:id/status", 
            put(routes::update_property_status)
        )
        
        // Routes investissements protégées par Bearer Token
        .route("/api/investments",
            get(routes::get_all_investments)
            .post(routes::create_investment)
        )
        .route("/api/investments/:id",
            get(routes::get_investment_by_id)
            .put(routes::update_investment)
            .delete(routes::delete_investment)
        )
        
        // Layers
        .layer(TraceLayer::new_for_http())
        .with_state(pool.clone());

    // Détermination de l'adresse d'écoute
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");

    println!("🚀 Server running on http://{}", addr);
    println!("📋 Routes disponibles:");
    println!("  - POST /auth/login (connexion par signature)");
    println!("  - POST /auth/logout (déconnexion)");
    println!("  - GET  /health (vérification santé)");
    println!("  - POST /users (création utilisateur)");
    println!("  - GET  /properties/public (propriétés validées - publique)");
    println!("  - GET  /api/properties (propriétés filtrées par rôle - Bearer Token requis)");
    println!("  - POST /api/properties (créer propriété - Manager/Admin Bearer Token)");
    println!("  - GET  /api/properties/:id (détail propriété - Bearer Token requis)");
    println!("  - PUT  /api/properties/:id (modifier propriété - Manager/Admin Bearer Token)");
    println!("  - PUT  /api/properties/:id/status (modifier statut - Admin Bearer Token uniquement)");
    println!("  - DELETE /api/properties/:id (supprimer propriété - Admin Bearer Token uniquement)");
    println!("  - GET  /api/investments (investissements filtrés par rôle - Bearer Token requis)");
    println!("  - POST /api/investments (créer investissement - Bearer Token requis)");
    println!("  - GET  /api/investments/:id (détail investissement - Bearer Token requis)");
    println!("  - PUT  /api/investments/:id (modifier investissement - Admin/Propriétaire Bearer Token)");
    println!("  - DELETE /api/investments/:id (supprimer investissement - Admin/Propriétaire Bearer Token)");

    // Démarrer le serveur
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}
