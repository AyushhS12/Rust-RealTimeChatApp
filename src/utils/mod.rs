use std::env;

use axum::{body::Body, extract::Request, http::request::Parts};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};

use crate::{
    db::Db,
    models::{Claims, User},
};

pub async fn extract_cookie(parts: Parts) -> Result<Claims, String>{
    let cookie = parts.headers.get("Cookie");
    match cookie {
        Some(cookie) => {
            let jwt = cookie.to_str().unwrap();
            let (_,token) = jwt.split_once("=").unwrap();
            let claims = validate_jwt(token);
            claims
        }
        None => Err("cookie invalid".to_string()),
    }
}

pub async fn extract_cookie_into_user(req: Request<Body>, db: &Db) -> Result<Option<User>, String> {
    let (parts, _) = req.into_parts();
    let claims = extract_cookie(parts).await;
    match claims {
                Ok(claims) => Ok(db.find_user_with_id(claims.sub).await),
                Err(e) => Err(e),
    }
}

pub fn validate_jwt(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").unwrap();
    // println!("{}",secret);
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    let token_data = jsonwebtoken::decode(token, &key, &validation);
    // println!("{:?}", token_data);
    match token_data {
        Ok(claims) => Ok(claims.claims),
        Err(e) => Err(e.to_string()),
    }
}
