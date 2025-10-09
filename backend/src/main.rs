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
    let address = match env::var("PORT") {
        Ok(p) => {
            format!("http://localhost:{}",p)
        }
        Err(e) => {
            log::error!("{}",e);
            String::from("http://localhost:7878")
        }
    };
    let _ = Server::new(address).await.listen().await;
}
