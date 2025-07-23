// db.rs

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use crate::models::UserRole;

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

// Fonction utilitaire pour obtenir le rôle d'un utilisateur par wallet
pub async fn get_user_role(pool: &PgPool, wallet: &str) -> UserRole {
    let role = sqlx::query!(
        r#"SELECT role as "role: UserRole" FROM users WHERE wallet = $1"#,
        wallet
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    role.map(|r| r.role).unwrap_or(UserRole::User)
}
