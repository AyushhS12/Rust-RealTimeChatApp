
use crate::{ routes::auth::*};
use axum::{routing::{get, post}, Router};
mod auth;

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