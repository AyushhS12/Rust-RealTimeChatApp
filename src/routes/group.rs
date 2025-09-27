use std::usize;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use log::error;
use mongodb::bson::{from_slice_utf8_lossy};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db::Db, utils::extract_cookie};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action")]
enum HandleMember {
    #[serde(rename = "add")]
    Add(AddMember),
    #[serde(rename = "remove")]
    Remove(RemoveMember),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AddMember {
    group_id: String,
    user_ids: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RemoveMember {
    group_id: String,
    user_ids: Vec<String>,
}

pub async fn add_or_remove_members(
    Extension(db): Extension<Db>,
    req: Request<Body>,
) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let id = extract_cookie(parts).await.unwrap().sub;
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let action = from_slice_utf8_lossy::<HandleMember>(&bytes);
    match action {
        Ok(HandleMember::Add(r)) => {
            let res = db
                .add_or_remove_members(id, r.group_id, r.user_ids, "add")
                .await;
            match res {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({
                        "success":true
                    })),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "err":e.error()
                    })),
                ),
            }
        }
        Ok(HandleMember::Remove(r)) => {
            let res = db
                .add_or_remove_members(id, r.group_id, r.user_ids, "remove")
                .await;
            match res {
                Ok(_) => (
                    StatusCode::OK,
                    Json(json!({
                        "success":true
                    })),
                ),
                Err(e) => (
                    StatusCode::BAD_REQUEST,
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
                    "err":e.to_string()
                })),
            )
        }
    }
}
