use std::sync::Arc;

use axum::{ middleware, Extension, Router};
use tokio::net::TcpListener;

use crate::{db::Db, middleware::auth_middleware, routes::*, thread_pool::ThreadPool};

pub struct Server {
    addr: &'static str,
    db:Arc<Db>
}

impl Server {
    pub async fn new(addr: &'static str) -> Self {
        Server { addr,db:Arc::new(Db::init().await.unwrap()) }
    }
    pub async fn listen(self) {
        let listener = TcpListener::bind(self.addr).await.unwrap();
        let db = self.db.clone();
        let app = self.manage_routers().await.layer(Extension(db));
        axum::serve(listener, app).await.unwrap();
    }

    async fn manage_routers(self) -> Router{
        let mut router = Router::new();
        router = router.nest("/auth", handle_auth_routes().await);
        router = router.nest("/api", handle_api_routes().await).layer(middleware::from_fn_with_state(self.db,auth_middleware));
        router
    }
}
