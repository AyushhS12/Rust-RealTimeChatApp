use crate::server::Server;
mod server;
mod routes;
mod models;
mod db;
mod middleware;
#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let server = Server::new("127.0.0.1:7878").await.listen();
    server.await
}
