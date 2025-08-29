use std::{env, sync::Arc, usize};

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{from_str, from_value, json, Value};

use crate::{db::Db, models::*};

pub async fn signup(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let (_, body) = req.into_parts();
    let data = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(data.as_ref()).into_owned();
    let val: User = from_str(&data).unwrap();
    println!("{:?}", val);
    let res = db.create_user(&val).await;
    match res {
        Ok(id) => Json(json!({
            "inserted_id":id
        })),
        Err(e) => {
            println!("{}", e.to_string());
            Json(json!({
                "err":"cannot create account try again"
            }))
        }
    }
}

pub async fn login(
    Extension(db): Extension<Arc<Db>>,
    Json(body): Json<Value>,
    req:&mut Request<Body>,
) -> impl IntoResponse {
    let data: LoginUser = from_value(body).unwrap();
    let user = db.find_user_with_email(data.email).await;
    match user {
        Some(u) => {
            let claims = &Claims { sub: u, exp: None };
            let secret = env::var("JWT_SECRET").unwrap();
            let key = &EncodingKey::from_secret(secret.as_ref());
            let token = encode(&Header::new(jsonwebtoken::Algorithm::HS256), claims, key);
            let res = req.headers_mut().append(
                "X-Authorization",
                HeaderValue::from_str(token.unwrap().as_str()).unwrap(),
            );
            if res {
                (
                    StatusCode::OK,
                    Json(json!({
                        "success":res
                    })),
                )
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "success":res
                    })),
                )
            }
        }
        None => (
            StatusCode::NOT_ACCEPTABLE,
            Json(json!({
                "success":false,
                "err":"user not found"
            })),
        ),
    }
}
