use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub wallet: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Property {
    pub id: Uuid,
    pub onchain_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub created_by: Option<Uuid>,
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
