mod error;
mod handlers;
mod state;

use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

pub use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .route("/rooms", post(handlers::rooms::create_room))
        .route("/rooms/:code/join", post(handlers::rooms::join_room))
        .route("/rooms/:code/start-game", post(handlers::rooms::start_game))
        .route("/rooms/:code/topic", post(handlers::rooms::submit_topic))
        .route("/rooms/:code/start-round", post(handlers::rooms::start_round))
        .route("/rooms/:code/guess", post(handlers::rooms::submit_guess))
        .route("/rooms/:code/events", get(handlers::sse::sse_handler))
        .route("/wiki/random", get(handlers::wiki::random_article))
        .layer(cors)
        .with_state(state)
        .fallback_service(ServeDir::new("static").fallback(ServeFile::new("static/index.html")));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
