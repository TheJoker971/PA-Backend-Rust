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
