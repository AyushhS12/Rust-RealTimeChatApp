use crate::{db::Db, models::*, utils::extract_cookie_into_user};
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
use cookie::{time::Duration as Samay, Cookie, CookieBuilder};
use jsonwebtoken::{encode, EncodingKey, Header};
// use lettre::AsyncTransport;
// use lettre::{
//     message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
//     Message, Tokio1Executor,
// };
use log::{error, info};
// use rand::Rng;
use serde_json::{from_str, from_value, json, Value};
use std::{
    env,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
    usize,
};

// Tried many things for the code to be able to work on render but it does not let me use the smtp protocol

// async fn send_request_for_email(user: &User) -> Result<usize, MyError> {
//     let from = "Glooo App <postmaster@sandbox72cf25da0d794b63a98eb7a837426b7b.mailgun.org>";
//     let to = format!("{} <{}>",user.name,user.email);
//     let subject = "OTP for verification on glooo";
//     let otp = rand::thread_rng().gen_range(100001..=999998);
//     let html = format!(
//         "<h4>OTP for signing up for Glooo is {}\
//     </h4><br><h3>Glooo - Stick People Together</h3>\
//     <br><br><p>if you didn't request this then ignore this email</p>",
//         otp
//     );
//     let client = reqwest::Client::new();
//     let key = env::var("MAILGUN_API_KEY").unwrap_or_else(|_|{"API_KEY".to_string()});
//     let params = [
//         ("from", from),
//         ("to", &to),
//         ("subject", subject),
//         ("html", &html),
//     ];
//     match client.post("https://api.mailgun.net/v3/sandbox72cf25da0d794b63a98eb7a837426b7b.mailgun.org/messages")
//         .basic_auth("api", Some(key))
//         .form(&params).send().await{
//             Ok(r) => {
//                 // debug!("{:?}",r);
//                 Ok(otp)
//             }
//             Err(e) => {
//                 let err = MyError::from_error(&e, "auth: otp func");
//                 Err(err)
//             }
//         }
// }

// async fn send_otp_email(user: &User) -> Result<usize, MyError> {
//     let receiver = format!("{} <{}>", user.name, user.email);
//     let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
//     let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
//     let smtp_server = env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
//     let sender = format!("Gloop Team <{}>", smtp_username);
//     let otp = rand::thread_rng().gen_range(100001..=999998);
//     let body = format!("<h4>OTP for signing up for Glooo is {}<h4><br><h3>Glooo - Stick People Together<h3><br><br><p>if you didn't request this then ignore this email<p>",otp);
//     let email = Message::builder()
//         .from(sender.parse().expect("Failed to parse from address"))
//         .to(receiver.parse().expect("Failed to parse to address"))
//         .subject("Signup For Glooo")
//         // Set the body of the email.
//         // This can be plain text or HTML.
//         .header(ContentType::TEXT_HTML)
//         .body(body)
//         .expect("Failed to build email message");

//     let creds = Credentials::new(smtp_username.to_owned(), smtp_password.to_owned());

//     let transporter = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
//         .unwrap()
//         .credentials(creds)
//         .build();
//     println!("Sending email...");
//     match transporter.send(email).await {
//         Ok(_) => {
//             println!("Email sent successfully!");
//             Ok(otp)
//         }
//         Err(e) => {
//             error!("Error sending email: {:?}", e);
//             Err(MyError::new(
//                 e.to_string(),
//                 "auth : verify email".to_string(),
//             ))
//         }
//     }
// }

pub async fn signup(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let (_, body) = req.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let data = String::from_utf8_lossy(&bytes).into_owned();
    let mut val: User = from_str(&data).unwrap();
    val.verified = false;
    // let otp1 = send_otp_email(&val).await.unwrap();
    // let otp = send_request_for_email(&val).await;
    // info!("{:?}", otp);
    let res = db.create_user(&mut val).await;
    match res {
        Ok(id) => {
            // let result = db.store_otp(otp.unwrap(), val.email).await;
            info!("{}", id);
            (
                StatusCode::OK,
                Json(json!({
                    "inserted_id":id
                })),
            )
        }
        Err(e) => {
            info!("{}", e.to_string());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "err":e.to_string()
                })),
            )
        }
    }
}

pub async fn login(
    Extension(db): Extension<Arc<Db>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let data: LoginUser = from_value(body).unwrap();
    let user = db.login_user(&data).await;
    match user {
        Some(u) => {
            let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                + Duration::from_secs(28 * 24 * 3600);
            let claims = &Claims {
                sub: u.id.unwrap().to_hex(),
                exp: exp.as_secs() as usize,
            };
            let secret = env::var("JWT_SECRET").unwrap();
            let key = &EncodingKey::from_secret(secret.as_bytes());
            let token = encode(&Header::new(jsonwebtoken::Algorithm::HS256), claims, key);
            let mut headers = HeaderMap::new();
            let t = token.unwrap();
            let value = format!("jwt={}; HttpOnly; Path=/;SameSite=None;Secure;", t.clone());
            headers.append(header::SET_COOKIE, HeaderValue::from_str(&value).unwrap());
            let _ = db.update_last_login(data.email).await;
            (
                StatusCode::OK,
                headers,
                Json(json!({
                    "success":true,
                    "token":t,
                    "verified":u.verified
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

pub async fn logout(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let (mut parts, _) = req.into_parts();
    let res = extract_cookie_into_user(&parts, &db).await;
    match res {
        Ok(Some(mut user)) => {
            let res = parts.headers.remove("Cookie");
            info!("deleted {:?}", res);
            let cookie = CookieBuilder::build(
                Cookie::build(("jwt", ""))
                    .path("/")
                    .max_age(Samay::ZERO)
                    .http_only(true)
                    .same_site(cookie::SameSite::None)
                    .secure(true),
            );
            let mut header = HeaderMap::new();
            header.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
            (
                StatusCode::OK,
                header,
                Json(json!({
                    "logout":"success",
                    "user": user.hide_pass()
                })),
            )
        }
        _ => {
            error!("cannot logout, error in auth : logout");
            (StatusCode::BAD_REQUEST, HeaderMap::new(), Json(json!({})))
        }
    }
}

pub async fn session(Extension(db): Extension<Arc<Db>>, req: Request<Body>) -> impl IntoResponse {
    let user = extract_cookie_into_user(&req.into_parts().0, &db).await;
    match user {
        Ok(Some(u)) => (
            StatusCode::OK,
            Json(json!({
                "success":true,
                "id":u.id
            })),
        ),
        _ => (
            StatusCode::NOT_ACCEPTABLE,
            Json(json!({
                "success":false
            })),
        ),
    }
}
