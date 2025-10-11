use log::{info};
use std::{env};

use crate::server::Server;

mod db;
mod middleware;
mod models;
mod routes;
mod server;
mod utils;
#[tokio::main]
async fn main() {
    match env::var("ENV") {
        Ok(env_name) => {
            match env_name.as_str() {
                "development" => {
                    dotenv::dotenv().ok();
                    println!("🧩 Running in development mode (loaded .env)");
                }
                "production" => {
                    println!("🚀 Running in production mode (using system env vars)");
                }
                other => {
                    println!("⚙️ Unknown ENV='{}', assuming development", other);
                    dotenv::dotenv().ok();
                }
            }
        }
        Err(_) => {
            eprintln!("ENV variable not found — defaulting to development mode");
            dotenv::dotenv().ok();
        }
    }
    env_logger::init();
    let port = env::var("PORT").unwrap_or_else(|_|{"7878".to_string()});
    let address = format!("localhost:{}",port);
    info!("listening on port : {}", address);
    let _ = Server::new(address).await.listen().await;
}
