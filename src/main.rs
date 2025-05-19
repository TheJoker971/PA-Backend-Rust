use axum::Router;
use dotenvy::dotenv;
use std::env;
use tokio;
use tower_http::trace::TraceLayer;

mod db;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let _pool = db::init_db().await;

    let app = routes::create_router().layer(TraceLayer::new_for_http());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("🚀 Server running on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
