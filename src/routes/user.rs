use std::sync::Arc;
use futures::stream::StreamExt;
use axum::{
    body::{to_bytes, Body}, extract::Request, http::StatusCode, response::IntoResponse, Extension, Json,
};
use serde_json::{from_str, json};

use crate::{db::{Db, IntoObjectId},models::{Requests, User}, utils::{extract_cookie, extract_cookie_into_user}};

pub async fn profile(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let res = extract_cookie_into_user(req, &db).await;
    match res {
        Ok(u) => match u {
            Some(mut user) => (
                StatusCode::OK,
                Json(json!({
                    "user":user.protect_pass()
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
            println!("{}", e.to_string());
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
    let query = req.uri().query();
    match query {
        Some(q) => {
            println!("{}", q);
            let res = db.find_users_with_substring(q.split_once("=").unwrap().1.to_string()).await;
            match res {
                Ok(mut cursor) => {
                    let mut users:Vec<User> = vec![];
                    while let Some(Ok(user)) = cursor.next().await{
                        users.push(user);
                    }
                    (
                    StatusCode::OK,
                    Json(json!({
                        "user":users,
                    })),
                )},
                Err(e) => {
                    println!("{}",e.to_string());
                    (
                    StatusCode::OK,
                    Json(json!({
                        "err":"user not found"
                    })),
                )},
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

pub async fn add_friend(Extension(db): Extension<Arc<Db>>, r: Request<Body>) -> impl IntoResponse{
    let (parts, body) = r.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes).into_owned();
    let req = from_str::<Requests>(&data);
    match req {
        Ok(mut request) => {
            request.from_id = Some(extract_cookie(parts).await.unwrap().sub.into_object_id());
            if request.status == ""{
                let res = db.add_friend_request(request).await;
            }
        },
        Err(e)=>{
            println!("{}",e.to_string());
        }
    }
}