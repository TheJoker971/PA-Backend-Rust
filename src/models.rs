// models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub signature: String,
    pub name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Property {
    pub id: Uuid,
    pub onchain_id: String,
    pub name: String,
    pub location: String,
    pub property_type: String,
    pub description: Option<String>,
    pub total_price: Option<BigDecimal>,
    pub token_price: Option<BigDecimal>,
    pub annual_yield: Option<BigDecimal>,
    pub image_url: Option<String>,
    pub documents: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub is_validated: bool,
    pub validated_at: Option<DateTime<Utc>>,
    pub validated_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Investment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub property_id: Uuid,
    pub amount_eth: BigDecimal,
    pub shares: i32,
    pub tx_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub token: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

// Structures pour les requêtes API

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub signature: String,
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePropertyRequest {
    pub onchain_id: String,
    pub name: String,
    pub location: String,
    pub property_type: String,
    pub description: Option<String>,
    pub total_price: Option<BigDecimal>,
    pub token_price: Option<BigDecimal>,
    pub annual_yield: Option<BigDecimal>,
    pub image_url: Option<String>,
    pub documents: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvestmentRequest {
    pub property_id: Uuid,
    pub amount_eth: BigDecimal,
    pub shares: i32,
    pub tx_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatePropertyRequest {
    pub is_validated: bool,
}