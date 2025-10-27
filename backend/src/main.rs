use log::{debug,info};
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
    let address = match env::var("PORT"){
        Ok(port) => {
            format!("0.0.0.0:{}",port)
        }
        Err(e) => {
            debug!("ENVIRONMENT VAR port not found defaulting to 7878\nerror: {}",e);
            format!("localhost:7878")
        }
    };
    info!("listening on address : http://{}", address);
    let _ = Server::new(address).await.listen().await;
}
