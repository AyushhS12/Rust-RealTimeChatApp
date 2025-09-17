
use crate::routes::{api::get_friend_request, auth::*, chat::handle_websocket, user::{profile, search, send_req}};
use axum::{routing::{get, post}, Json, Router};
use serde_json::json;
mod auth;
mod user;
mod api;
pub mod chat;
// #[axum::debug_handler]
pub async fn handle_auth_routes() -> Router {
    let router = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login));
    router
}

pub async fn handle_api_routes() -> Router{
    let router = Router::new()
        .route("/hello", get(|| async {
            Json(json!({
                "msg":"Hello World"
            }))
        }))
        .route("/get_requests", get(get_friend_request));
    router
}

pub async fn handle_user_routes() -> Router {
    let router = Router::new()
        .route("/profile", get(profile))
        .route("/search", get(search))
        .route("/send_request", post(send_req));
    router
}
// #[axum::debug_handler]
pub async fn handle_chat_routes() -> Router{
    let router = Router::new()
        .route("/1", get(handle_websocket));
    router
}