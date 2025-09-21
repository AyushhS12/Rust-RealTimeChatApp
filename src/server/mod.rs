use std::{collections::HashMap, sync::Arc};

use axum::{ middleware, Extension, Router};
use mongodb::bson::oid::ObjectId;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{db::Db, middleware::auth_middleware, routes::{chat::{Client, Manager}, *}};

pub struct Server {
    addr: &'static str,
    db:Arc<Db>,
    manager:Arc<Mutex<Manager>>
}

pub type GroupManager = Arc<HashMap<ObjectId, HashMap<ObjectId, Client>>>;

impl Server {
    pub async fn new(addr: &'static str) -> Self {
        Server { addr,db:Arc::new(Db::init().await.unwrap()) , manager: Arc::new(Mutex::new(Manager::new()))}
    }
    pub async fn listen(self) {
        let listener = TcpListener::bind(self.addr).await.unwrap();
        let db = self.db.clone();
        let app = self.manage_routers().await.layer(Extension(db));
        axum::serve(listener, app).await.unwrap();
    }

    async fn manage_routers(self) -> Router{
        let mut router = Router::new();
        router = router.nest("/api", handle_api_routes());
        router = router.nest("/create", handle_create_routes());
        router = router.nest("/chat", handle_chat_routes()).layer(Extension(self.manager.clone())).layer(Extension(GroupManager::default()));
        router = router.nest("/user", handle_user_routes()).layer(middleware::from_fn_with_state(self.db, auth_middleware));
        router = router.nest("/auth", handle_auth_routes());
        router
    }
}
