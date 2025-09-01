use std::{sync::Arc};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{db::Db,utils::validate_jwt}; //, models::Claims};


pub async fn auth_middleware(
    State(db): State<Arc<Db>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, impl IntoResponse> {
    let header = req.headers().get("Cookie");
    match header {
        Some(h) => {
            // println!("{:?}",h);
            let auth = h.to_str().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "err":"please login"
                    })),
                )
            })?;
            let auth_token: &str;
            if let Some((name, token)) = auth.split_once("=") {
                if name == "jwt" {
                    auth_token = token;
                } else {
                    return Err((
                        StatusCode::NOT_ACCEPTABLE,
                        Json(json!({
                            "err":"invalid token string"
                        })),
                    ));
                }
            } else {
                return Err((
                    StatusCode::NOT_ACCEPTABLE,
                    Json(json!({
                        "err":"invalid token string 2"
                    })),
                ));
            }
            let result = validate_jwt(auth_token);
            match result {
                Ok(claims) => {
                    // println!("{}", id);
                    let user = claims.sub;
                    let res = db.find_user_with_id(user).await;
                    match res {
                        Some(_) => {
                            Ok(next.run(req).await)
                        }
                        None => {
                            return Err((
                                StatusCode::OK,
                                Json(json!({
                                    "success":true
                                })),
                            ));
                        }
                    }
                    // match user {
                    //     Some(_) => {
                    //         next.run(req).await;
                    //         Ok((
                    //             StatusCode::OK,
                    //             Json(json!({
                    //                 "success":true
                    //             })),
                    //         )
                    //             .into_response())
                    //     }
                    //     None => Err((
                    //         StatusCode::UNAUTHORIZED,
                    //         Json(json!({
                    //             "err":"user not found"
                    //         })),
                    //     )),
                    // }
                }
                Err(e) => {
                    println!("{}", e.to_string());
                    Err((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({
                            "err":e.to_string()
                        })),
                    ))
                }
            }
        }
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "err":"please login"
            })),
        )),
    }
}
