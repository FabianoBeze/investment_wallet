mod db;
mod models;
mod handlers;
mod services;

use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;
use sqlx::Row; // Importa o trait Row para usar o método get()
// use sqlx::MySqlPool; // Removido pois estava não utilizado

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::create_pool().await;
    
    // Tarefa em segundo plano para atualizar preços periodicamente (a cada 5 minutos)
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            // Busca todos os investimentos do sistema
            if let Ok(mut all_investments) = sqlx::query_as::<_, crate::models::Investment>(
                "SELECT * FROM investments"
            )
            .fetch_all(&pool_clone)
            .await {
                for inv in &mut all_investments {
                    if let Some(address) = &inv.token_address {
                        if let Ok(price) = crate::services::dex_screener::get_token_price(address).await {
                            // Atualiza no banco
                            let _ = sqlx::query(
                                "UPDATE investments SET current_price = ? WHERE id = ?"
                            )
                            .bind(price)
                            .bind(inv.id)
                            .execute(&pool_clone)
                            .await;
                        }
                    }
                }
                println!("Preços de todos os investimentos atualizados!");
            }
        }
    });
    
    // Cria uma tarefa em segundo plano para atualizar preços a cada 5 minutos
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // 30s = 5min
        loop {
            interval.tick().await;
            // Busca todos os investimentos do banco
            if let Ok(mut investments) = sqlx::query_as::<_, crate::models::Investment>(
                "SELECT * FROM investments"
            )
            .fetch_all(&pool_clone)
            .await {
                // Atualiza cada um
                for inv in &mut investments {
                    if let Some(address) = &inv.token_address {
                        if let Ok(price) = crate::services::dex_screener::get_token_price(address).await {
                            let _ = sqlx::query(
                                "UPDATE investments SET current_price = ? WHERE id = ?"
                            )
                            .bind(price)
                            .bind(inv.id)
                            .execute(&pool_clone)
                            .await;
                        }
                    }
                }
                // Calcula e salva o valor total da carteira para cada usuário
                if let Ok(user_totals) = sqlx::query(
                    "SELECT user_id, SUM(amount * current_price) as total FROM investments GROUP BY user_id"
                )
                .fetch_all(&pool_clone)
                .await {
                    for row in user_totals {
                        let user_id: i32 = row.get("user_id");
                        let total: f64 = row.get("total");
                        // Salva no histórico
                        let _ = sqlx::query(
                            "INSERT INTO portfolio_history (user_id, total_value) VALUES (?, ?)"
                        )
                        .bind(user_id)
                        .bind(total)
                        .execute(&pool_clone)
                        .await;
                    }
                }
                println!("Preços atualizados e histórico salvo!");
            }
        }
    });

    let app = Router::new()
        .route(
            "/",
            get(handlers::home_handler)
                .post(handlers::create_investment_handler),
        )
        .route(
            "/login",
            get(handlers::login_page_handler)
                .post(handlers::login_post_handler),
        )
        .route(
            "/delete/{id}",
            get(handlers::delete_investment_handler),
        )
        .route(
    "/logout",
    get(handlers::logout_handler),
        )
        .route(
    "/register",
    get(handlers::register_page_handler)
        .post(handlers::register_post_handler),
        )

        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Servidor rodando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}