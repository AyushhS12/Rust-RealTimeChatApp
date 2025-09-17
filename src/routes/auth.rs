use std::{
    env,sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}, usize
};

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{
        header::{self},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::IntoResponse,
    Extension, Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{from_str, from_value, json, Value};

use crate::{db::Db, models::*};

pub async fn signup(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let (_, body) = req.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes).into_owned();
    let mut val: User = from_str(&data).unwrap();
    let res = db.create_user(&mut val).await;
    match res {
        Ok(id) => (StatusCode::OK,Json(json!({
            "inserted_id":id
        }))),
        Err(e) => {
            println!("{}", e.to_string());
            (StatusCode::NOT_ACCEPTABLE,Json(json!({
                "err":e.to_string()
            })))
        }
    }
}

pub async fn login(
    Extension(db): Extension<Arc<Db>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let data: LoginUser = from_value(body).unwrap();
    let user = db.find_user_with_email(data.email.clone()).await;
    match user {
        Some(u) => {
            let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                + Duration::from_secs(28 * 24 * 3600);
            let claims = &Claims {
                sub: u.get_id().unwrap().to_hex(),
                exp: exp.as_secs() as usize,
            };
            let secret = env::var("JWT_SECRET").unwrap();
            let key = &EncodingKey::from_secret(secret.as_bytes());
            let token = encode(&Header::new(jsonwebtoken::Algorithm::HS256), claims, key);
            let mut headers = HeaderMap::new();
            let value = format!("jwt={}; HttpOnly; Path=/;", token.unwrap());
            headers.append(header::SET_COOKIE, HeaderValue::from_str(&value).unwrap());
            let _ = db.update_last_login(data.email).await;
            (
                StatusCode::OK,
                headers,
                Json(json!({
                    "success":true
                })),
            )
        }
        None => (
            StatusCode::NOT_ACCEPTABLE,
            HeaderMap::new(),
            Json(json!({
                "success":false,
                "err":"user not found"
            })),
        ),
    }
}
