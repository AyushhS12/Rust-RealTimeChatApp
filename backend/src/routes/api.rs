use axum::{
    body::{to_bytes, Body},
    extract::{Path, Request},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::{oid::ObjectId, Bson};
use log::{debug, error, info};
use std::{sync::Arc, usize};
// use mongodb::bson::oid::ObjectId;
use serde_json::{from_str, json};

use crate::{
    db::Db,
    models::{FriendReq, FriendRequest, MyError, Requests},
    utils::{extract_cookie, extract_cookie_into_user},
};

pub async fn get_chats(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let (parts, _) = req.into_parts();
    let jwt = extract_cookie(parts).await;
    match jwt {
        Ok(claims) => {
            let requests = db.get_chats(claims.sub).await;
            match requests {
                Ok(chats) => {
                    info!("{:?}",chats);
                    (
                    StatusCode::OK,
                    Json(json!({
                        "chats":chats
                    })),
                )
                },
                Err(e) => {
                    error!("error in get chats {}", e.to_string());
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

pub async fn get_messages(
    Extension(db): Extension<Arc<Db>>,
    Path(chat_id):Path<ObjectId>,
    _: Request<Body>,
) -> impl IntoResponse {
    let res = db.get_messages_with_chat_id(chat_id).await;

    match res {
        Ok(messages) => {
            (StatusCode::OK,Json(json!({
                "messages":messages
            })))
        }
        Err(e) => {
            error!("{}",e);
            (StatusCode::BAD_REQUEST,Json(json!({
                "err":"invalid id"
            })))
        }
    }
}

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
    let result: Result<(String, Bson), MyError>;
    match request {
        FriendRequest::Accept { from_id } => {
            result = db
                .handle_friend_request(Some(id), Some(from_id), "accept")
                .await;
        }
        FriendRequest::Reject { from_id } => {
            result = db
                .handle_friend_request(Some(id), Some(from_id), "reject")
                .await;
        }
    }
    match result {
        Ok(msg) => {
            info!("{:?}", msg);
            (
                StatusCode::OK,
                Json(json!({
                    "success":true,
                    "message":msg.0,
                    "inserted_id":msg.1
                })),
            )
        }
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

pub async fn get_my_id(Extension(db): Extension<Arc<Db>>,req: Request<Body>) -> impl IntoResponse{
    let (parts , _) = req.into_parts();
    let user = extract_cookie_into_user(&parts, &db).await;
    match user {
        Ok(Some(u)) => {
            Json(json!({
                "id":u.id
            }))
        }
        Ok(None) => {
            Json(json!({
                "err":"user not found"
            }))
        }
        Err(e) => {
            error!("{}",e);
            Json(json!({
                "err":e
            }))
        }
    }

}

pub async fn handle_incoming_request(Extension(db): Extension<Arc<Db>>,req: Request<Body>) -> impl IntoResponse{
    let (parts , body) = req.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes).to_string();
    let id = extract_cookie(parts).await.unwrap().sub;
    let req = from_str::<FriendReq>(&data).unwrap();
    let request = Requests::new_from_friend_req(req, id);
    let res = db.add_friend_request(request).await;
    match res {
        Ok(msg) => {
            debug!("{}",msg);
            (StatusCode::OK, Json(json!({
                "success":true
            })))
        }
        Err(e) => {
            error!("{}",e.to_string());
            (StatusCode::OK, Json(json!({
                "success":false
            })))
        }
    }
}