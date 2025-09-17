use std::sync::Arc;

use axum::{body::Body, extract::Request, http::StatusCode, response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::{db::Db, utils::extract_cookie};


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
