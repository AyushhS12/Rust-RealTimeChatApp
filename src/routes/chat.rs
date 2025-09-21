use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket}, State, WebSocketUpgrade
    },
    http::Request,
    response::IntoResponse,
    Error, Extension,
};
use futures::{SinkExt, StreamExt};
use mongodb::bson::{self, oid::ObjectId};
use serde_json::{from_str, to_string};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};

use crate::{
    db::{Db, IntoObjectId}, models::Message as Msg, server::GroupManager, utils::extract_cookie_for_ws
};
#[derive(Clone)]
pub struct Client {
    sender: mpsc::UnboundedSender<Msg>,
    active: bool,
}

pub struct Manager {
    clients: HashMap<String, Client>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            clients: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Client) {
        self.clients.insert(key, value);
    }

    pub fn remove(&mut self, c: String) {
        self.clients.remove(&c);
    }

    pub fn find(&self, c: &String) -> Option<&Client> {
        self.clients.get(c)
    }
}

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(manager): Extension<Arc<Mutex<Manager>>>,
    Extension(db): Extension<Arc<Db>>,
    req: Request<Body>,
) -> impl IntoResponse {
    ws.on_failed_upgrade(|err: axum::Error| {
        println!("error :{}", err.to_string());
    })
    .on_upgrade(|ws| async move {
        let (parts, _) = req.into_parts();
        let cookie = extract_cookie_for_ws(parts).await;
        match cookie {
            Some(id) => {
                // println!("{:?}", id);
                handle_chat(manager.clone(), id, ws, db).await;
            }
            None => (),
        }
    })
}

async fn handle_chat(manager: Arc<Mutex<Manager>>, id: String, ws: WebSocket, db: Arc<Db>) {
    println!("Websocket connection established");
    //splitting socket
    let (s, mut receiver) = ws.split();
    let sender = Arc::new(Mutex::new(s));
    let (tx, mut rx) = mpsc::unbounded_channel::<Msg>();
    let c = Client {
        sender: tx,
        active: true,
    };
    manager.lock().await.insert(id.clone(), c);

    let db_rx = Arc::clone(&db);
    let manager_rx = Arc::clone(&manager);
    let sender_rx = Arc::clone(&sender);

    tokio::spawn(async move {
        loop {
            match receiver.next().await {
                Some(Ok(msg)) => {
                    if let Ok(data) = msg.to_text() {
                        if let Ok(mut message) = from_str::<Msg>(data) {
                            let client = {
                                let mgr = manager_rx.lock().await;
                                mgr.find(&message.to_id.clone().unwrap().to_hex()).cloned()
                            };
                            message.from_id = Some(id.clone().into_object_id());
                            let mut sender = sender_rx.lock().await;
                            match client {
                                Some(c) => {
                                    message.created_at = Some(bson::DateTime::now());
                                    let r = c.sender.send(message.clone());
                                    match r {
                                        Ok(()) => {
                                            let res =
                                                db_rx.add_message_to_db(message.clone()).await;
                                            println!("{:?}", res);
                                            sender.send(Message::text("hello")).await.unwrap();
                                        }
                                        Err(e) => {
                                            sender
                                                .send(Message::text(String::from(
                                                    "Message cannot be sent",
                                                )))
                                                .await
                                                .unwrap();
                                            println!("{}", e.to_string())
                                        }
                                    }
                                }
                                None => {
                                    let res = db_rx.add_message_to_db(message).await;
                                    println!("{:?}", res);
                                }
                            }
                        } else {
                            println!("kuch aur dikkat hai");
                        }
                    } else {
                        println!("kuch dikkat hai");
                    }
                }
                Some(Err(e)) => {
                    println!("{:?}", e);
                }
                None => {
                    manager_rx.lock().await.remove(id);
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut sender = sender.lock().await;
            sender
                .send(Message::text(to_string(&msg).unwrap()))
                .await
                .unwrap();
        }
    });
}

pub async fn handle_group_connection(
    ws: WebSocketUpgrade,
    Extension(manager): Extension<Arc<Mutex<Manager>>>,
    Extension(group_manager): Extension<GroupManager>,
    Extension(db): Extension<Arc<Db>>,
    req: Request<Body>,
) -> impl IntoResponse {
    ws.on_failed_upgrade(|err: Error| println!("{}", err.to_string()))
        .on_upgrade(|ws| async move {
            let (parts, _) = req.into_parts();
            let cookie = extract_cookie_for_ws(parts.clone()).await;
            match cookie {
                Some(id) => {
                    // println!("{:?}", id);
                    let group_id = parts.uri.query();
                    handle_group_chat(group_id.unwrap(), id, ws, db).await;
                }
                None => (),
            }
        })
}

struct Group {
    id: Option<ObjectId>,
    members: HashSet<String>,
}

async fn handle_group_chat(group_id: &str, id: String, ws: WebSocket, db: Arc<Db>) {
    let res = db.find_group(group_id.to_string()).await;
    match res {
        Some(group_id) => {
            let (mut sender, mut receiver) = ws.split();
            tokio::spawn(async move {
                loop {
                    match receiver.next().await {
                        Some(Ok(msg)) => {
                            if let Ok(data) = msg.to_text() {
                                if let Ok(message) = from_str::<Msg>(data) {}
                            }
                        }
                        Some(Err(e)) => {
                            println!("{}", e.to_string());
                        }
                        None => {
                            break;
                        }
                    }
                }
            });
        }
        None => {
            println!("group Not found");
            return;
        }
    }
}
