// scripts/migrate_to_supabase.rs
//
// Script pour migrer les données d'une base PostgreSQL existante vers Supabase
// Utilisation : cargo run --bin migrate_to_supabase

use dotenvy::dotenv;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, PgPool, Row};
use std::env;
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct User {
    id: Uuid,
    wallet: String,
    email: Option<String>,
    name: Option<String>,
    password_hash: String,
    role: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct Property {
    id: Uuid,
    onchain_id: i32,
    name: String,
    description: Option<String>,
    image_url: Option<String>,
    category: Option<String>,
    created_by: Option<Uuid>,
}

#[derive(Debug, FromRow)]
struct Investment {
    id: Uuid,
    user_id: Uuid,
    property_id: Uuid,
    amount_eth: f64,
    shares: i32,
    tx_hash: String,
    created_at: chrono::NaiveDateTime,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Charger les variables d'environnement
    dotenv().ok();

    // Source DB (ancienne base de données)
    let source_url = env::var("SOURCE_DATABASE_URL").expect("SOURCE_DATABASE_URL must be set");
    let source_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&source_url)
        .await?;

    // Target DB (Supabase)
    let target_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let target_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&target_url)
        .await?;

    println!("🔄 Début de la migration vers Supabase...");

    // 1. Migrer les utilisateurs
    migrate_users(&source_pool, &target_pool).await?;

    // 2. Migrer les propriétés
    migrate_properties(&source_pool, &target_pool).await?;

    // 3. Migrer les investissements
    migrate_investments(&source_pool, &target_pool).await?;

    println!("✅ Migration terminée avec succès !");

    Ok(())
}

async fn migrate_users(source: &PgPool, target: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("👤 Migration des utilisateurs...");

    // Récupérer les utilisateurs de la source
    let users = sqlx::query_as::<_, User>(
        "SELECT id, wallet, email, name, password_hash, role, created_at FROM users",
    )
    .fetch_all(source)
    .await?;

    println!("  📊 {} utilisateurs trouvés", users.len());

    // Insérer les utilisateurs dans la cible
    for user in users {
        // Insérer l'utilisateur
        sqlx::query!(
            r#"INSERT INTO users (id, wallet, email, name, password_hash, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (id) DO UPDATE SET
                  wallet = EXCLUDED.wallet,
                  email = EXCLUDED.email,
                  name = EXCLUDED.name,
                  password_hash = EXCLUDED.password_hash,
                  created_at = EXCLUDED.created_at"#,
            user.id,
            user.wallet,
            user.email,
            user.name,
            user.password_hash,
            user.created_at
        )
        .execute(target)
        .await?;

        // Insérer le rôle (en utilisant les 8 premiers caractères du wallet)
        let wallet_short = user.wallet.chars().take(8).collect::<String>();
        sqlx::query!(
            r#"INSERT INTO roles (wallet_short, role)
               VALUES ($1, $2)
               ON CONFLICT (wallet_short) DO UPDATE SET
                  role = EXCLUDED.role"#,
            wallet_short,
            user.role
        )
        .execute(target)
        .await?;
    }

    println!("  ✅ Migration des utilisateurs terminée");
    Ok(())
}

async fn migrate_properties(source: &PgPool, target: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("🏠 Migration des propriétés...");

    // Récupérer les propriétés de la source
    let properties = sqlx::query_as::<_, Property>(
        "SELECT id, onchain_id, name, description, image_url, category, created_by FROM properties",
    )
    .fetch_all(source)
    .await?;

    println!("  📊 {} propriétés trouvées", properties.len());

    // Insérer les propriétés dans la cible
    for property in properties {
        // Déterminer le type de propriété en fonction de la catégorie
        let property_type = match property.category.as_deref() {
            Some("villa") | Some("apartment") | Some("house") => "Residential",
            Some("office") | Some("retail") | Some("mall") => "Commercial",
            Some("warehouse") | Some("factory") => "Industrial",
            _ => "Residential", // Par défaut
        };

        // Insérer la propriété avec des valeurs par défaut pour les nouveaux champs
        sqlx::query!(
            r#"INSERT INTO properties (
                id, onchain_id, name, location, type, description, 
                image_url, documents, created_by, is_validated, created_at
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
               ON CONFLICT (id) DO UPDATE SET
                  onchain_id = EXCLUDED.onchain_id,
                  name = EXCLUDED.name,
                  location = EXCLUDED.location,
                  type = EXCLUDED.type,
                  description = EXCLUDED.description,
                  image_url = EXCLUDED.image_url,
                  documents = EXCLUDED.documents,
                  created_by = EXCLUDED.created_by"#,
            property.id,
            property.onchain_id,
            property.name,
            "Location non spécifiée", // Valeur par défaut pour le nouveau champ location
            property_type,
            property.description,
            property.image_url,
            serde_json::Value::Array(vec![]), // Documents vides par défaut
            property.created_by,
            true // Toutes les propriétés existantes sont considérées comme validées
        )
        .execute(target)
        .await?;
    }

    println!("  ✅ Migration des propriétés terminée");
    Ok(())
}

async fn migrate_investments(source: &PgPool, target: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("💰 Migration des investissements...");

    // Récupérer les investissements de la source
    let investments = sqlx::query_as::<_, Investment>(
        "SELECT id, user_id, property_id, amount_eth, shares, tx_hash, created_at FROM investments",
    )
    .fetch_all(source)
    .await?;

    println!("  📊 {} investissements trouvés", investments.len());

    // Insérer les investissements dans la cible
    for investment in investments {
        sqlx::query!(
            r#"INSERT INTO investments (id, user_id, property_id, amount_eth, shares, tx_hash, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                  user_id = EXCLUDED.user_id,
                  property_id = EXCLUDED.property_id,
                  amount_eth = EXCLUDED.amount_eth,
                  shares = EXCLUDED.shares,
                  tx_hash = EXCLUDED.tx_hash,
                  created_at = EXCLUDED.created_at"#,
            investment.id,
            investment.user_id,
            investment.property_id,
            investment.amount_eth,
            investment.shares,
            investment.tx_hash,
            investment.created_at
        )
        .execute(target)
        .await?;
    }

    println!("  ✅ Migration des investissements terminée");
    Ok(())
} 