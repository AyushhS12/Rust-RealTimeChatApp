use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    http::Request,
    response::IntoResponse,
    Extension,
};
use bson::DateTime;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use log::{debug, error, info};
use mongodb::results::InsertOneResult;
use serde_json::{from_str, json, to_string};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use crate::{db::{Db, IntoObjectId}, models::ChatMessage, utils::extract_cookie_for_ws};
#[derive(Clone)]
pub struct Client {
    sender: mpsc::UnboundedSender<ChatMessage>,
    _active: bool,
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
        error!("error :{}", err.to_string());
    })
    .on_upgrade(|ws| async move {
        let (parts, _) = req.into_parts();
        let cookie = extract_cookie_for_ws(parts).await;
        match cookie {
            Some(id) => {
                info!("{}", id);
                handle_chat(manager.clone(), id, ws, db).await;
            }
            None => (),
        }
    })
}

async fn handle_chat(manager: Arc<Mutex<Manager>>, id: String, ws: WebSocket, db: Arc<Db>) {
    debug!("Websocket connection established");
    // ========== Splitting socket ==========
    let (s, mut receiver) = ws.split();
    // For sharing sender concurrently ==========
    let sender = Arc::new(Mutex::new(s));
    // ========== Local channels to transfer data among different threads
    let (tx, mut rx) = mpsc::unbounded_channel::<ChatMessage>();
    // ========== Adding client to the map ========== ==========
    let c = Client {
        sender: tx,
        _active: true,
    };
    manager.lock().await.insert(id.clone(), c);
    // Cloning DB
    let db_rx = Arc::clone(&db);
    let manager_rx = Arc::clone(&manager);
    let sender_rx = Arc::clone(&sender);
    // ========== Readloop ==========

    let readloop = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let mut sender = sender.lock().await;
            sender
                .send(Message::text(to_string(&msg).unwrap()))
                .await
                .unwrap();
        }
    });

    // ========== Write loop ==========
    tokio::spawn(async move {
        loop {
            match receiver.next().await {
                Some(Ok(msg)) => {
                    if let Ok(data) = msg.to_text() {
                        if let Ok(message) = from_str::<ChatMessage>(data) {
                            match message {
                                ChatMessage::Direct(mut m) => {
                                    m.created_at = Some(DateTime::now());
                                    if !db.chat_exists(m.chat_id.clone().unwrap()).await {
                                        error!("Chat does not exists");
                                        return;
                                    }
                                    let client = {
                                        let mgr = manager_rx.lock().await;
                                        mgr.find(&m.to_id.clone().unwrap().to_hex()).cloned()
                                    };
                                    m.from_id = Some(id.clone().into_object_id());
                                    let sender = sender_rx.clone();
                                    match client {
                                        Some(c) => {
                                            debug!("user found");
                                            c.sender.send(ChatMessage::Direct(m.clone())).unwrap();
                                            let result = db_rx
                                                .add_message_to_db(ChatMessage::Direct(m.clone()))
                                                .await;
                                            match_result(sender, result).await;
                                        }
                                        None => {
                                            let result = db_rx
                                                .add_message_to_db(ChatMessage::Direct(m.clone()))
                                                .await;
                                            match_result(sender, result).await;
                                        }
                                    }
                                }
                                ChatMessage::Group(mut m) => {
                                    m.created_at = Some(DateTime::now());
                                    if !db.group_exists(m.group_id.clone().unwrap()).await {
                                        error!("Group does not exists");
                                        return;
                                    }
                                    m.from_id = Some(id.clone().into_object_id());
                                    let group =
                                        db_rx.find_group(m.group_id.unwrap()).await.unwrap();
                                    let mgr = manager_rx.lock().await;
                                    for member in group.members {
                                        let sender = sender_rx.clone();
                                        let user = mgr.find(&member.to_hex());
                                        match user {
                                            Some(u) => {
                                                #[allow(warnings)]
                                                u.sender.send(ChatMessage::Group(m.clone()));
                                                let result = db_rx
                                                    .add_message_to_db(ChatMessage::Group(
                                                        m.clone(),
                                                    ))
                                                    .await;
                                                match_result(sender, result).await;
                                            }
                                            None => {
                                                let result = db_rx
                                                    .add_message_to_db(ChatMessage::Group(
                                                        m.clone(),
                                                    ))
                                                    .await;
                                                match_result(sender, result).await;
                                            }
                                        }
                                    }
                                }
                            };
                        } else {
                            error!("kuch dikkat hai");
                        }
                    } else {
                        error!("kuch aur dikkat hai");
                    }
                }
                Some(Err(e)) => {
                    error!("{}", e.to_string());
                }
                None => {
                    info!("Shutting down readloop for {}", id);
                    manager_rx.lock().await.remove(id);
                    readloop.abort();
                    break;
                }
            }
        }
    });
}

async fn match_result(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    result: Option<InsertOneResult>,
) {
    match result {
        Some(r) => {
            info!("database response : [{}]", r.inserted_id);
        }
        None => {
            error!("failed to add the message");
            #[allow(warnings)]
            sender.lock().await.send(Message::text(
                json!({
                    "err":"unable to send message"
                })
                .to_string(),
            ));
        }
    }
}
