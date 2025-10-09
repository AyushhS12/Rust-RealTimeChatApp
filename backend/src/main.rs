use std::env;

use crate::server::Server;
mod server;
mod routes;
mod models;
mod db;
mod middleware;
mod utils;
#[tokio::main]
async fn main() {
    if env::var("ENV").unwrap() != "production".to_string() {
        dotenv::dotenv().unwrap();
    }
    env_logger::init();
    let _ = Server::new("127.0.0.1:7878").await.listen().await;
}
