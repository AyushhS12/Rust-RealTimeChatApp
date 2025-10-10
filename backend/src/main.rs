use std::env;

use log::info;

use crate::server::Server;
mod server;
mod routes;
mod models;
mod db;
mod middleware;
mod utils;
#[tokio::main]
async fn main() {
    env_logger::init();
    match env::var("ENV") {
        Ok(v) => {
            if v == "development"{
                dotenv::dotenv().unwrap();
            }
        }
        Err(e) => {
            log::error!("{}",e);
            dotenv::dotenv().unwrap();
        }
    };
    let address = match env::var("PORT") { 
        Ok(p) => {
            format!("http://localhost:{}",p)
        }
        Err(e) => {
            log::error!("{}",e);
            String::from("http://localhost:7878")
        }
    };
    info!("listening on port : {}",address);
    let _ = Server::new(address).await.listen().await;
}
