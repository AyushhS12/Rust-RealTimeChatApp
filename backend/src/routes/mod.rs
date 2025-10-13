use axum::{routing::{get, post}, Json, Router};
use serde_json::json;
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
        .route("/login", post(auth::login))
        .route("/logout", get(auth::logout))
        .route("/session", get(auth::session));
    // .route("/verify_user", post(user::verify))
    router
}

pub fn handle_api_routes() -> Router{
    let router = Router::new()
        .route("/hello", get(|| async {
            Json(json!({
                "msg":"Hello World"
            }))
        }))
        .nest("/requests",api_request_routes())
        .nest("/chat", api_chat_routes())
        .route("/get_my_id", get(api::get_my_id));
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
        .route("/chat", post(create::handle_chat_creation));
    router
}

pub fn handle_group_routes() -> Router{
    let router = Router::new()
        .route("/manage_members", post(group::add_or_remove_members));
    router
}

// Api Nested Routes

fn api_request_routes() -> Router{
    let router = Router::new()
        .route("/get_requests", get(api::get_friend_request))
        .route("/handle_request", post(api::handle_friend_request));
    router
}

fn api_chat_routes() -> Router {
    Router::new()
        .route("/get_chats", get(api::get_chats))
        .nest("/message", api_messages_routes())
}

fn api_messages_routes() -> Router{
    Router::new()
        .route("/get_messages/{chat_id}", get(api::get_messages))
}