use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use log::error;
use std::{sync::Arc, usize};
// use mongodb::bson::oid::ObjectId;
use serde_json::{from_str, json};

use crate::{db::Db, models::{FriendRequest, MyError}, utils::extract_cookie};

pub async fn get_friend_request(
    Extension(db): Extension<Arc<Db>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let (parts, _) = req.into_parts();
    let jwt = extract_cookie(parts).await;
    match jwt {
        Ok(claims) => {
            let requests = db.fetch_user_friend_request(claims.sub).await;
            match requests {
                Ok(r) => (
                    StatusCode::OK,
                    Json(json!({
                        "requests":r
                    })),
                ),
                Err(e) => {
                    println!("error in get friend request {}", e.to_string());
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "err":e.to_string()
                        })),
                    )
                }
            }
        }
        Err(e) => {
            println!("error in get friend requests 2 {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "err":e
                })),
            )
        }
    }
}

pub async fn handle_friend_request(
    Extension(db): Extension<Arc<Db>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let id = extract_cookie(parts).await.unwrap().sub;
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes);
    let request = from_str::<FriendRequest>(&data).unwrap();
    let result: Result<(), MyError>;
    match request {
        FriendRequest::Accept { to_id } => {
            result = db
                .handle_friend_request(Some(id), Some(to_id), "accept")
                .await;
        }
        FriendRequest::Reject { to_id } => {
            result = db
                .handle_friend_request(Some(id), Some(to_id), "reject")
                .await;
        }
    }
    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success":true
            })),
        ),
        Err(e) => {
            error!("{}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "err":e.error()
                })),
            )
        }
    }
}
