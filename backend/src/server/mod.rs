use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{http::{header, Method}, middleware, Extension, Router};
use mongodb::bson::oid::ObjectId;
use tokio::{net::TcpListener, sync::Mutex, time};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{
    db::Db,
    middleware::auth_middleware,
    routes::{
        chat::{Client, Manager},
        *,
    },
};

pub struct Server {
    addr: String,
    db: Arc<Db>,
    manager: Arc<Mutex<Manager>>,
    group_man: GroupManager,
}

pub type GroupManager = Arc<HashMap<ObjectId, HashMap<ObjectId, Client>>>;

impl Server {
    pub async fn new(addr: String) -> Self {
        Server {
            addr,
            db: Arc::new(Db::init().await.unwrap()),
            manager: Arc::new(Mutex::new(Manager::new())),
            group_man: GroupManager::default(),
        }
    }
    pub async fn listen(self) {
        let listener = TcpListener::bind(self.addr.clone()).await.unwrap();
        let db = self.db.clone();
        let allowed_origins = AllowOrigin::list(["http://localhost:5173".parse().unwrap()]);

        let cors = CorsLayer::new()
            // 1. Allow the specific origin of your React app
            .allow_origin(allowed_origins)
            // 2. CRITICAL: Allow credentials (cookies) to be sent
            .allow_credentials(true)
            // 3. Allow common HTTP methods
            .allow_methods([Method::GET, Method::POST,Method::DELETE])
            // 4. Allow specific headers that might be sent in a request
            .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE]);
        let app = self
            .manage_routers()
            .await
            .layer(Extension(db.clone()))
            .layer(cors);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10 * 60));
            log::info!("OTP cleanup task started. Begin cleanup again in 10 minutes");
            loop {
                interval.tick().await;
                let db = db.clone();
                match db.check_and_clear_otps().await {
                    Ok(s) => {
                        log::info!("========================================================================\n{}", s);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
        });
        axum::serve(listener, app).await.unwrap();
    }

    async fn manage_routers(self) -> Router {
        let mut router = Router::new();
        router = router.nest("/api", handle_api_routes());
        router = router.nest("/create", handle_create_routes());
        router = router.nest("/group", handle_group_routes());
        router = router
            .nest("/chat", handle_chat_routes())
            .layer(Extension(self.manager.clone()))
            .layer(Extension(self.group_man.clone()));
        router = router
            .nest("/user", handle_user_routes())
            .layer(middleware::from_fn_with_state(self.db, auth_middleware));
        router = router.nest("/auth", handle_auth_routes());
        router
    }
}
