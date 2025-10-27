use crate::{
    db::{Db, IntoObjectId},
    models::{FriendReq, Requests, User},
    utils::{extract_cookie, extract_cookie_into_user},
};
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use futures::stream::StreamExt;
use log::error;
use serde_json::{from_str, json};
use std::{sync::Arc, usize};

pub async fn profile(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let res = extract_cookie_into_user(&req.into_parts().0, &db).await;
    match res {
        Ok(u) => match u {
            Some(mut user) => (
                StatusCode::OK,
                Json(json!({
                    "user":user.hide_pass()
                })),
            ),
            None => (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "err":"user not found"
                })),
            ),
        },
        Err(e) => {
            error!("{}", e.to_string());
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "err":"login first then come here"
                })),
            )
        }
    }
}

pub async fn search<T>(Extension(db): Extension<Arc<Db>>, req: Request<T>) -> impl IntoResponse {
    let (parts, _) = req.into_parts();
    let uri = parts.uri.clone();
    let query = uri.query();
    match query {
        Some(q) => {
            let (query, value) = q.split_once("=").unwrap();
            if query != "user" {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "err":"invalid query"
                    })),
                );
            }
            let id = extract_cookie(parts).await.unwrap().sub;
            let res = db.find_users_with_substring(value.to_string()).await;
            match res {
                Ok(mut cursor) => {
                    let mut users: Vec<User> = vec![];
                    while let Some(Ok(mut user)) = cursor.next().await {
                        if user.id.unwrap().to_hex() == id {
                            continue;
                        }
                        users.push(user.hide_pass());
                    }
                    (
                        StatusCode::OK,
                        Json(json!({
                            "users":users,
                        })),
                    )
                }
                Err(e) => {
                    error!("{}", e.to_string());
                    (
                        StatusCode::OK,
                        Json(json!({
                            "err":"user not found"
                        })),
                    )
                }
            }
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "err":"invalid query"
            })),
        ),
    }
}

pub async fn send_req(Extension(db): Extension<Arc<Db>>, r: Request<Body>) -> impl IntoResponse {
    let (parts, body) = r.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes).into_owned();
    let req = from_str::<FriendReq>(&data);
    match req {
        Ok(r) => {
            let from_id = extract_cookie(parts).await.unwrap().sub;
            let request = Requests::new_from_friend_req(r, from_id);
            let res = db.add_friend_request(request).await;
            match res {
                Ok(id) => (
                    StatusCode::OK,
                    Json(json!({
                        "inserted_id":id
                    })),
                ),
                Err(e) => (
                    StatusCode::NOT_ACCEPTABLE,
                    Json(json!({
                        "err":e.error()
                    })),
                ),
            }
        }
        Err(e) => {
            error!("{}", e.to_string());
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                        "err":"invalid request"
                })),
            )
        }
    }
}





// Only auth function which is in users routes because of my poor memory

// pub async fn verify(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
//     let (_, body) = req.into_parts();
//     let bytes = to_bytes(body, usize::MAX).await.unwrap();
//     let data = String::from_utf8_lossy(&bytes);
//     let otp = from_str::<Otp>(&data).unwrap();
//     let res = db.update_verification_for_user(otp).await;
//     match res {
//         Ok(msg) => {
//             info!("{}",msg);
//             Json(json!({
//                 "success":true,
//                 "message":"user verified successfully"
//             }))
//         }
//         Err(e) => {
//             error!("{}",e);
//             let err = format!("cannot verify user , {}", e.error());
//             Json(json!({
//                 "success":false,
//                 "err":err
//             }))
//         }
//     }
// }

// pub async fn _handle_friend_request(
//     Extension(db): Extension<Arc<Db>>,
//     req: Request<Body>,
// ) -> impl IntoResponse {
//     let (parts , body) = req.into_parts();
//     let claims = extract_cookie(parts).await.unwrap();
//     let data = to_bytes(body, usize::MAX).await.unwrap();
//     let mut request = from_str::<RequestHandler>(String::from_utf8_lossy(&data).into_owned().as_str()).unwrap();
//     request.from_id = Some(claims.sub.into_object_id());
//     let res = db.handle_friend_request(request).await;
//     match res {
//         Ok(()) => {
//             (StatusCode::OK, Json(json!({
//                 "success":true
//             })))
//         },
//         Err(e)=> {
//             println!("{}",e);
//             (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
//                 "err":e
//             })))
//         }
//     }
// }
