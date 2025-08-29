use std::{env, sync::Arc};

use axum::{
    body::Body, extract::{Request, State}, http::StatusCode, middleware::Next, response::{IntoResponse, Response},
    Extension, Json,
};
use jsonwebtoken::{DecodingKey, Validation};
use mongodb::bson::{oid::ObjectId};
use serde_json::json;

use crate::{db::Db, models::User};

fn validate_jwt(token: &str) -> Result<Option<ObjectId>, String> {
    let secret = env::var("JWT_SECRET").unwrap();
    let key = &DecodingKey::from_secret(secret.as_ref());
    let validation = &Validation::default();
    match jsonwebtoken::decode::<User>(token, key, validation) {
        Ok(claims) => Ok(claims.claims.get_id()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn auth_middleware(
    State(db): State<Arc<Db>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let header = req.headers().get("X-Authorization");
    match header {
        Some(h) => {
            let auth = h.to_str().map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "err":"please login"
                    })),
                )
            })?;
            let result = validate_jwt(auth);
            match result {
                Ok(id) => match id {
                    Some(i) => {
                        match db.find_user_with_id(i).await {
                            Some(_) => {
                                next.run(req).await;
                                Ok((StatusCode::OK,Json(json!({
                                    "msg":"login success"
                                }))).into_response())
                            }
                            None => Err((StatusCode::UNAUTHORIZED, Json(json!({
                                "err":"please login 2"
                            })))),
                        }
                    },
                    None =>{
                        Err((StatusCode::UNAUTHORIZED,Json(json!({
                            "err":"login please 2"
                        }))))
                    }
                },
                Err(e) => {
                    println!("{}", e.to_string());
                    Err((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({
                            "err":"login please"
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
