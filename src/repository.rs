use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::MySqlPool;

use crate::{
    models::{Investment, User},
    state::AppState,
};

pub struct Repository {
    db: MySqlPool,
}

impl Repository {
    pub async fn get_user_by_name(
        &self,
        username: &str,
    ) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_investments(
        &self,
        user_id: i32,
    ) -> sqlx::Result<Vec<Investment>> {
        sqlx::query_as::<_, Investment>(
            "SELECT * FROM investments WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_investment(
        &self,
        user_id: i32,
        asset_name: &str,
        amount: f64,
        current_price: f64,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO investments
             (user_id, asset_name, amount, current_price)
             VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(asset_name)
        .bind(amount)
        .bind(current_price)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn delete_investment(
        &self,
        investment_id: i32,
        user_id: i32,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "DELETE FROM investments
             WHERE id = ?
             AND user_id = ?"
        )
        .bind(investment_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> impl std::future::Future<
        Output = Result<Self, Self::Rejection>
    > + Send {

        let db = state.db.clone();

        async move {
            Ok(Self { db })
        }
    }
}