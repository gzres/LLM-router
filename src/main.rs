mod config;
mod model;
mod router;

use axum::{Router, routing::get, routing::post};
use config::load_config;
use model::{AppState, refresh_models_loop};
use router::{forward_completion, forward_request, healthz, list_models, main_page};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = load_config("config.yml");
    let state = AppState::new(config);

    let state_clone = state.clone();
    tokio::spawn(async move {
        refresh_models_loop(state_clone).await;
    });

    let app = Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(forward_request))
        .route("/v1/completions", post(forward_completion))
        .route("/healthz", get(healthz))
        .route("/", get(main_page))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Starting server on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
