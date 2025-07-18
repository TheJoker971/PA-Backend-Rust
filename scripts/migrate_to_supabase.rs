// scripts/migrate_to_supabase.rs

use sqlx::PgPool;
use dotenvy::dotenv;
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger les variables d'environnement
    dotenv().ok();

    // Récupérer l'URL de la base de données
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL doit être définie dans le fichier .env");

    // Connexion à la base de données
    let pool = PgPool::connect(&database_url).await?;

    println!("🔄 Exécution de la migration vers Supabase...");

    // Lire le fichier de migration
    let migration_sql = fs::read_to_string("migrations/supabase_migration.sql")
        .expect("Impossible de lire le fichier migrations/supabase_migration.sql");

    // Exécuter chaque instruction SQL séparément
    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            match sqlx::query(trimmed).execute(&pool).await {
                Ok(_) => println!("✅ Instruction SQL exécutée avec succès"),
                Err(e) => println!("❌ Erreur lors de l'exécution de l'instruction SQL: {}", e),
            }
        }
    }

    println!("✅ Migration terminée avec succès!");
    Ok(())
} 