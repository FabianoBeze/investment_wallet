use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::env;

pub async fn create_pool() -> MySqlPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL deve estar configurada");
    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Falha ao conectar ao MySQL")
}