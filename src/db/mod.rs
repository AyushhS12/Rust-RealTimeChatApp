use crate::models::*;
use futures::StreamExt;
use log::error;
use mongodb::error::Error;
use mongodb::results::InsertOneResult;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, DateTime},
    options::FindOptions,
    Client, Collection, Cursor,
};
use std::collections::HashSet;
use std::{env, str::FromStr, sync::Arc};
pub trait IntoObjectId {
    fn into_object_id(self) -> ObjectId;
    fn to_string(self) -> String;
}

#[derive(Clone)]
pub struct Db {
    users: Arc<Collection<User>>,
    friends: Arc<Collection<Friend>>,
    chats: Arc<Collection<Chat>>,
    messages: Arc<Collection<DirectMessage>>,
    groups: Arc<Collection<Group>>,
    requests: Arc<Collection<Requests>>,
    group_messages: Arc<Collection<GroupMessage>>,
}

impl IntoObjectId for String {
    fn into_object_id(self) -> ObjectId {
        ObjectId::from_str(&self).unwrap()
    }
    fn to_string(self) -> String {
        self
    }
}

impl IntoObjectId for ObjectId {
    fn into_object_id(self) -> ObjectId {
        self
    }
    fn to_string(self) -> String {
        self.to_hex()
    }
}

impl Db {
    pub async fn init() -> Result<Self, String> {
        let uri = env::var("DATABASE_URL").unwrap();
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("rust");
        let ping_res = db.run_command(bson::doc! {"ping":1}).await;
        match ping_res {
            Ok(doc) => {
                println!("{:?}", doc);
                let users = Arc::new(db.collection::<User>("users"));
                let friends = Arc::new(db.collection::<Friend>("friends"));
                let chats = Arc::new(db.collection::<Chat>("chats"));
                let messages = Arc::new(db.collection::<DirectMessage>("messages"));
                let groups = Arc::new(db.collection::<Group>("groups"));
                let requests = Arc::new(db.collection::<Requests>("requests"));
                let group_messages = Arc::new(db.collection::<GroupMessage>("group_messages"));
                Ok(Db {
                    users,
                    friends,
                    chats,
                    messages,
                    groups,
                    requests,
                    group_messages,
                })
            }
            Err(e) => {
                println!("{}", e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn find_user_with_id(&self, id: impl IntoObjectId) -> Option<User> {
        let res = self.users.find_one(doc! {"_id":id.into_object_id()}).await;
        match res {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e.to_string());
                None
            }
        }
    }

    pub async fn find_user_with_email(&self, email: String) -> Option<User> {
        let filter = doc! {
            "email":email
        };
        self.users.find_one(filter).await.unwrap()
    }

    pub async fn update_last_login(&self, email: String) -> Result<(), String> {
        let filter = doc! {
            "email":email.clone()
        };
        let res = self.users.find_one(filter.clone()).await.unwrap();
        match res {
            Some(_) => {
                let res = self
                    .users
                    .update_one(
                        filter,
                        doc! {
                            "$set":{
                                "last_login":Bson::DateTime(DateTime::now()),
                            }
                        },
                    )
                    .await;
                match res {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
            None => Err(String::from("invalid id in update last login")),
        }
    }

    pub async fn create_user(&self, user: &mut User) -> Result<Bson, String> {
        let res = self
            .users
            .find_one(doc! {"username":user.username.clone()})
            .await;
        match res {
            Ok(Some(_)) => {
                return Err(String::from("user already exists with this username"));
            }
            Ok(None) => (),
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
        let res = self.users.find_one(doc! {"email":user.email.clone()}).await;
        match res {
            Ok(Some(_)) => {
                return Err(String::from("user already exists with this email"));
            }
            Ok(None) => (),
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
        user.created_at = Some(DateTime::now());
        let res = self.users.insert_one(user).await;
        match res {
            Ok(doc) => Ok(doc.inserted_id),
            Err(e) => {
                println!("{}", e.to_string());
                Err(e.to_string())
            }
        }
    }
    pub async fn find_users_with_substring(&self, name: String) -> Result<Cursor<User>, Error> {
        let filter = doc! {
            "username":{
                "$regex":name,
                "$options":"i"
            },
        };
        let find_options = FindOptions::builder().limit(5).build();
        let cursor = self.users.find(filter).with_options(find_options).await;
        cursor
    }
    // ========== Chats Collection ==========

    pub async fn create_chat<T: IntoObjectId>(
        &self,
        first: T,
        second: T,
    ) -> Result<InsertOneResult, String> {
        let users = Vec::from([first.into_object_id(), second.into_object_id()]);
        let filter = doc! {
            "users":doc! {
                "$all":users.clone()
            }
        };
        let res = self.chats.find_one(filter).await;
        match res {
            Ok(Some(_)) => {
                println!("Chat already exists");
                Err("Chat already exists".to_string())
            }
            Ok(None) => {
                let chat = Chat::new(users);
                let res = self.chats.insert_one(chat).await;
                match res {
                    Ok(r) => Ok(r),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    // ========== Friends Collection ==========
    pub async fn find_friend_request<Y>(
        &self,
        from_id: Option<Y>,
        to_id: Option<Y>,
    ) -> Option<Requests>
    where
        Y: IntoObjectId,
    {
        let filter = doc! {
            "$and":[
                {"from_id": from_id.unwrap().into_object_id()},
                {"to_id":to_id.unwrap().into_object_id()}
            ]
        };
        let res = self.requests.find_one(filter).await;
        match res {
            Ok(o) => o,
            Err(e) => {
                println!("{}", e.to_string());
                None
            }
        }
    }

    pub async fn fetch_user_friend_request<I>(&self, id: I) -> Result<Vec<Requests>, Error>
    where
        I: IntoObjectId,
    {
        let filter = doc! { "to_id":id.into_object_id()};
        let res = self.requests.find(filter).await;
        match res {
            Ok(mut cursor) => {
                let mut requests: Vec<Requests> = vec![];
                while let Some(Ok(req)) = cursor.next().await {
                    requests.push(req);
                }
                Ok(requests)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn add_friend_request(&self, req: Requests) -> Result<Bson, String> {
        let r = self.find_friend_request(req.from_id, req.to_id).await;
        match r {
            Some(_) => Err(String::from(
                "request already exists cannot make a duplicate",
            )),
            None => {
                let res = self.requests.insert_one(req).await;
                match res {
                    Ok(i) => Ok(i.inserted_id),
                    Err(e) => {
                        println!("add friend err: {}", e.to_string());
                        Err(e.to_string())
                    }
                }
            }
        }
    }

    pub async fn handle_friend_request(&self, request: RequestHandler) -> Result<(), String> {
        let res = self
            .find_friend_request(request.from_id, request.to_id)
            .await;
        match res {
            Some(req) => {
                if req.valid_status() {
                    match request.action.as_str() {
                        "accept" => {
                            let doc = Friend::new(req.from_id, req.to_id);
                            let res = self.friends.insert_one(doc).await;
                            match res {
                                Ok(res) => {
                                    println!("added friend in db : {:?}", res);
                                    Ok(())
                                }
                                Err(e) => {
                                    println!("{}", e.to_string());
                                    Err(e.to_string())
                                }
                            }
                        }
                        "reject" => {
                            let query = doc! {
                                "$and":{
                                    "from_id":req.from_id,
                                    "to_id":req.to_id
                                }
                            };
                            let res = self.requests.delete_one(query).await;
                            match res {
                                Ok(id) => {
                                    println!("deleted : {:?}", id);
                                    Ok(())
                                }
                                Err(e) => {
                                    println!("{}", e.to_string());
                                    Err(e.to_string())
                                }
                            }
                        }
                        _ => Err(String::from("inavlid request action")),
                    }
                } else {
                    Err(String::from("invalid request status"))
                }
            }
            None => Err(String::from("invalid request in handle friend request")),
        }
    }
    // ========== Messages Collection ==========
    pub async fn add_message_to_db(&self, msg: ChatMessage) -> Option<InsertOneResult> {
        match msg {
            ChatMessage::Direct(msg) => {
                if msg.from_id == msg.to_id {
                    return None;
                }
                let res = self.messages.insert_one(&msg).await;
                match res {
                    Ok(r) => {
                        let query = doc! {
                            "_id":msg.chat_id,
                        };
                        let update = doc! {
                            "$push":doc! {
                                "messages":r.inserted_id.clone()
                            }
                        };
                        let res = self.chats.update_one(query, update).await.unwrap();
                        println!("{:?}", res);
                        Some(r)
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        None
                    }
                }
            },
            ChatMessage::Group(m) => {
                let res = self.group_messages.insert_one(m).await;
                match res {
                    Ok(r) => {
                        Some(r)
                    }
                    Err(e) => {
                        error!("{}",e.to_string());
                        None
                    }
                }
            }
        }
    }

    //========== Group Collection ==========
    pub async fn find_group(&self, group_id: impl IntoObjectId) -> Option<Group> {
        let id = group_id.into_object_id();
        if let Ok(Some(group)) = self
            .groups
            .find_one(doc! {
                "_id":id
            })
            .await
        {
            Some(group)
        } else {
            None
        }
    }

    pub async fn create_group_chat<T: IntoObjectId>(
        &self,
        id: T,
        members: HashSet<T>,
    ) -> Option<InsertOneResult> {
        let mut admin = HashSet::new();
        admin.insert(id.into_object_id());
        let mut users: HashSet<ObjectId> = HashSet::new();
        for user in members {
            users.insert(user.into_object_id());
        }
        let group = Group::new(admin, users);
        let res = self.groups.insert_one(group).await;
        match res {
            Ok(r) => Some(r),
            Err(e) => {
                println!("{}", e.to_string());
                None
            }
        }
    }
    // pub async fn users(self) -> Arc<Collection<User>> {
    //     self.users.clone()
    // }
    // pub async fn chats(self) -> Arc<Collection<Chat>> {
    //     self.chats.clone()
    // }
    // pub async fn messages(self) -> Arc<Collection<Message>> {
    //     self.messages.clone()
    // }
    // pub async fn requests(self) -> Arc<Collection<Requests>> {
    //     self.requests.clone()
    // }
    // pub async fn groups(self) -> Arc<Collection<Group>> {
    //     self.groups.clone()
    // }
    // pub async fn groups_messages(self) -> Arc<Collection<GroupMessage>> {
    //     self.group_messages.clone()
    // }
}
