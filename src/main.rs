use axum::{Router, serve};
use dotenvy::dotenv;
use std::env;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

mod db;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let _pool = db::init_db().await;

    let app = routes::create_router().layer(TraceLayer::new_for_http());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("🚀 Server running on http://{}", addr);

    serve(listener, app).await.unwrap();
}
