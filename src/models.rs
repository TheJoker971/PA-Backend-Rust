// models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

// Enum pour le statut des propriétés
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "property_status", rename_all = "lowercase")]
pub enum PropertyStatus {
    Pending,
    Validated,
    Rejected,
}

impl std::fmt::Display for PropertyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyStatus::Pending => write!(f, "pending"),
            PropertyStatus::Validated => write!(f, "validated"),
            PropertyStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl From<String> for PropertyStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "validated" => PropertyStatus::Validated,
            "rejected" => PropertyStatus::Rejected,
            _ => PropertyStatus::Pending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub signature: String,
    pub name: Option<String>,
    pub role: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Property {
    pub id: Uuid,
    pub onchain_id: String,
    pub name: String,
    pub location: String,
    pub property_type: String,  // Mappé depuis la colonne "type"
    pub description: Option<String>,
    pub total_price: BigDecimal,  // NOT NULL dans la DB
    pub token_price: BigDecimal,  // NOT NULL dans la DB  
    pub annual_yield: BigDecimal, // NOT NULL dans la DB
    pub image_url: Option<String>,
    pub documents: Option<Vec<String>>,
    pub created_by: Uuid,         // NOT NULL dans la DB
    pub created_at: DateTime<Utc>,
    pub status: Option<PropertyStatus>,
    pub status_updated_at: Option<DateTime<Utc>>,
    pub status_updated_by: Option<Uuid>,
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
    pub total_price: BigDecimal,        // Requis
    pub token_price: BigDecimal,        // Requis  
    pub annual_yield: BigDecimal,       // Requis
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
pub struct UpdateInvestmentRequest {
    pub amount_eth: BigDecimal,
    pub shares: i32,
    pub tx_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePropertyStatusRequest {
    pub status: PropertyStatus,
    pub comment: Option<String>, // Optionnel : commentaire pour le changement de statut
}