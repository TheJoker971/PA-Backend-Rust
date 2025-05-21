use axum::{Router, serve};
use dotenvy::dotenv;
use std::env;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use sqlx::PgPool;
use std::fs;

mod db;
mod routes;

#[tokio::main]
async fn main() {
    // Charger les variables d'environnement
    dotenv().ok();

    // Connexion à la base de données
    let pool: PgPool = db::init_db().await;

    // Lire le fichier SQL de migration
    let schema_sql = fs::read_to_string("migrations/schema.sql")
        .expect("Failed to read schema.sql");

    // Exécuter chaque instruction SQL séparément
    for statement in schema_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("Failed to execute: {}\nError: {}", trimmed, e));
        }
    }

    // Création du routeur
    let app: Router = routes::create_router(pool).layer(TraceLayer::new_for_http());

    // Détermination de l'adresse d'écoute
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    // Création du listener TCP
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("🚀 Server running on http://{}", addr);

    // Démarrage du serveur
    serve(listener, app).await.unwrap();
}
