use axum::{routing::get, Router};
use crate::AppState;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/challenge", get(|| async { "Challenge endpoint" }))
        .route("/authenticate", get(|| async { "Authenticate endpoint" }))
}

pub fn governance_routes() -> Router<AppState> {
    Router::new()
        .route("/proposals", get(|| async { "Proposals endpoint" }))
        .route("/votes", get(|| async { "Votes endpoint" }))
}

pub fn websocket_routes() -> Router<AppState> {
    Router::new()
        .route("/governance", get(|| async { "WebSocket endpoint" }))
}

async fn health_check() -> &'static str {
    "OK"
}