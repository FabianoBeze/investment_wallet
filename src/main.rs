mod db;
mod models;
mod handlers;

use axum::{routing::get, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pool = db::create_pool().await;

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