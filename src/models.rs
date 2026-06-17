use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Investment {
    pub id: i32,
    pub asset_name: String,
    pub amount: f64,
    pub current_price: f64,
    pub user_id: i32,
    pub token_address: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // User ID
    pub exp: usize,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct PortfolioHistoryEntry {
    pub id: i32,
    pub user_id: i32,
    pub total_value: f64,
    pub recorded_at: Option<chrono::DateTime<chrono::Utc>>,
}