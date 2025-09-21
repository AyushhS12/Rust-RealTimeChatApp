use axum::{routing::{get, post}, Json, Router};
use serde_json::json;

use crate::{routes::group::handle_chat_creation, server::GroupManager};
mod auth;
mod user;
mod api;
pub mod chat;
mod group;
// #[axum::debug_handler]
pub fn handle_auth_routes() -> Router {
    let router = Router::new()
        .route("/signup", post(auth::signup))
        .route("/login", post(auth::login));
    router
}

pub fn handle_api_routes() -> Router{
    let router = Router::new()
        .route("/hello", get(|| async {
            Json(json!({
                "msg":"Hello World"
            }))
        }))
        .route("/get_requests", get(api::get_friend_request));
    router
}

pub fn handle_user_routes() -> Router {
    let router = Router::new()
        .route("/profile", get(user::profile))
        .route("/search", get(user::search))
        .route("/send_request", post(user::send_req));
    router
}
// #[axum::debug_handler]
pub fn handle_chat_routes() -> Router{
    let router = Router::new()
        .route("/", get(chat::handle_websocket))
        .route("/group", get(chat::handle_group_connection));
    router
}

pub fn handle_create_routes() -> Router{
    let router = Router::new()
        .route("/group", post(group::handle_group_creation))
        .route("/chat", post(handle_chat_creation));
    router
}