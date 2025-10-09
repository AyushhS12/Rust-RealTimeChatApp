use std::{sync::Arc, usize};

use axum::{
    body::{to_bytes, Body}, extract::Request, http::StatusCode, response::IntoResponse, Extension, Json
};
use log::{error};
use serde_json::{from_slice, from_str, json};

use crate::{db::Db, models::{ChatRequest, Members}, utils::extract_cookie};

pub async fn handle_group_creation(
    Extension(db): Extension<Arc<Db>>,
    req: Request<Body>,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let id = extract_cookie(parts).await.unwrap().sub;
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = from_slice::<Members>(&bytes).unwrap();
    let res = db.create_group_chat(id, data.members).await;
    match res {
        Some(r) => Json(json!({
            "group_id":r.inserted_id,
            "success":true
        })),
        None => {
            error!("unable to create group");
            Json(json!({
                "success":false
            }))
        }
    }
}

pub async fn handle_chat_creation(Extension(db): Extension<Arc<Db>>,req: Request<Body>) -> impl IntoResponse{
    let (parts, body) = req.into_parts();
    let id = extract_cookie(parts).await.unwrap().sub;
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let s = String::from_utf8_lossy(&bytes);
    let second = from_str::<ChatRequest>(&s).unwrap();
    let chat = db.create_chat(id, second.second.unwrap().to_hex()).await;
    match chat {
        Ok(r) => {
            (StatusCode::OK,Json(json!({
                "id":r.inserted_id,
                "success":true
            })))
        }
        Err(e) => {
            error!("{}", e.to_string());
            (StatusCode::BAD_REQUEST, Json(json!({
                "err":"inavlid request body"
            })))
        }
    }
}