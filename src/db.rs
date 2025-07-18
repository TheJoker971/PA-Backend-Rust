// db.rs

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::time::Duration;

pub async fn init_db() -> PgPool {
    // Récupérer l'URL de connexion à Supabase
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    // Créer le pool de connexions avec des options avancées
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .expect("Failed to connect to Supabase database")
}

// Fonction utilitaire pour obtenir le rôle d'un wallet
pub async fn get_wallet_role(pool: &PgPool, wallet: &str) -> String {
    // Extraire les 8 premiers caractères du wallet
    let wallet_short = wallet.chars().take(8).collect::<String>();
    
    // Récupérer le rôle
    sqlx::query_scalar!(
        r#"SELECT role FROM roles WHERE wallet_short = $1"#,
        wallet_short
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
    .unwrap_or_else(|| "user".to_string())
}
