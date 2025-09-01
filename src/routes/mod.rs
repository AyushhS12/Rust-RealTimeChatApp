
use crate::routes::{auth::*, user::{profile, search}};
use axum::{routing::{get, post}, Router};
mod auth;
mod user;
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
            "Hello World".to_string()
        }));
    router
}

pub async fn handle_user_routes() -> Router {
    let router = Router::new()
        .route("/profile", get(profile))
        .route("/search", get(search));
    router
}