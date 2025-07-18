// models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDateTime, DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub signature: String,
    pub name: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub wallet_short: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Property {
    pub id: Uuid,
    pub onchain_id: i32,
    pub name: String,
    pub location: String,
    pub property_type: String,
    pub description: Option<String>,
    pub total_price: Option<f64>,
    pub token_price: Option<f64>,
    pub annual_yield: Option<f64>,
    pub image_url: Option<String>,
    pub documents: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub is_validated: bool,
    pub validated_at: Option<NaiveDateTime>,
    pub validated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Investment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Uuid,
    pub amount_eth: f64,
    pub shares: i32,
    pub tx_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub token: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

// Structures pour les requêtes API

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithRole {
    pub id: Uuid,
    pub wallet: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub signature: String,
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePropertyRequest {
    pub onchain_id: i32,
    pub name: String,
    pub location: String,
    pub property_type: String,
    pub description: Option<String>,
    pub total_price: Option<f64>,
    pub token_price: Option<f64>,
    pub annual_yield: Option<f64>,
    pub image_url: Option<String>,
    pub documents: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvestmentRequest {
    pub property_id: Uuid,
    pub amount_eth: f64,
    pub shares: i32,
    pub tx_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub signature: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatePropertyRequest {
    pub is_validated: bool,
}