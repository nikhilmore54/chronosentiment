// mod core; // Removed as core is now a crate
mod dto;
mod errors;
mod handlers;
mod routes;
mod services;
mod strategy_id_parse;

use axum::{routing::get, Router};
use services::evaluation_service::EvaluationService;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    
    let evaluation_service = EvaluationService::new();

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // Build our application with a route
    let app = Router::new()
        .route("/", get(|| async { "Hello, ChronoSentiment API!" }))
        .merge(routes::strategy_routes::strategy_routes())
        .with_state(evaluation_service)
        .layer(cors);

    // Run our app with hyper, listening globally on port 8000
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to 0.0.0.0:8000: {}", e);
            return;
        }
    };
    if let Ok(addr) = listener.local_addr() {
        println!("listening on {}", addr);
    }
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
    }
}
