mod error;
mod handlers;
mod state;

use std::sync::Arc;
use axum::{Router, routing::{get, post}, http::header};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

pub use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Content-hashed Vite assets: immutable for 1 year
    let assets_service = ServeDir::new("static/assets")
        .append_index_html_on_directories(false);

    // All other static files (index.html, icons): must revalidate
    let static_service = ServeDir::new("static")
        .fallback(ServeFile::new("static/index.html"));

    let app = Router::new()
        .route("/rooms", post(handlers::rooms::create_room))
        .route("/rooms/:code/join", post(handlers::rooms::join_room))
        .route("/rooms/:code/start-game", post(handlers::rooms::start_game))
        .route("/rooms/:code/topic", post(handlers::rooms::submit_topic))
        .route("/rooms/:code/start-round", post(handlers::rooms::start_round))
        .route("/rooms/:code/guess", post(handlers::rooms::submit_guess))
        .route("/rooms/:code/events", get(handlers::sse::sse_handler))
        .route("/wiki/random", get(handlers::wiki::random_article))
        .nest_service(
            "/assets",
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable".parse::<header::HeaderValue>().unwrap(),
                ))
                .service(assets_service),
        )
        .layer(cors)
        .with_state(state)
        .fallback_service(
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    "no-cache".parse::<header::HeaderValue>().unwrap(),
                ))
                .service(static_service),
        );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
