use axum::{routing::{get, post}, Json, Router};
use serde_json::json;

use crate::routes::{create::handle_chat_creation, group::add_or_remove_members};
mod auth;
mod user;
mod api;
pub mod chat;
mod create;
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
        .nest("/requests",api_request_routes());
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
        .route("/", get(chat::handle_websocket));
    router
}

pub fn handle_create_routes() -> Router{
    let router = Router::new()
        .route("/group", post(create::handle_group_creation))
        .route("/chat", post(handle_chat_creation));
    router
}

pub fn handle_group_routes() -> Router{
    let router = Router::new()
        .route("/manage_members", post(add_or_remove_members));
    router
}

// Api Nested Routes

fn api_request_routes() -> Router{
    let router = Router::new()
        .route("/get_requests", get(api::get_friend_request))
        .route("/handle_request", post(api::handle_friend_request));
    router
}