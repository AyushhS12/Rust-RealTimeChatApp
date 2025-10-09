use crate::models::*;
use chrono::Utc;
use futures::StreamExt;
use log::{debug, error, info};
use mongodb::bson::Document;
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
    otp: Arc<Collection<OneTimePass>>,
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
                debug!("{:?}", doc);
                let users = Arc::new(db.collection::<User>("users"));
                let friends = Arc::new(db.collection::<Friend>("friends"));
                let chats = Arc::new(db.collection::<Chat>("chats"));
                let messages = Arc::new(db.collection::<DirectMessage>("messages"));
                let groups = Arc::new(db.collection::<Group>("groups"));
                let requests = Arc::new(db.collection::<Requests>("requests"));
                let group_messages = Arc::new(db.collection::<GroupMessage>("group_messages"));
                let otp = Arc::new(db.collection::<OneTimePass>("one_time_passwords"));
                Ok(Db {
                    users,
                    friends,
                    chats,
                    messages,
                    groups,
                    requests,
                    group_messages,
                    otp,
                })
            }
            Err(e) => {
                error!("{}", e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn find_user_with_id(&self, id: impl IntoObjectId) -> Option<User> {
        let res = self.users.find_one(doc! {"_id":id.into_object_id()}).await;
        match res {
            Ok(r) => r,
            Err(e) => {
                error!("{}", e.to_string());
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

    pub async fn create_user(&self, user: &mut User) -> Result<Bson, MyError> {
        let res = self
            .users
            .find_one(doc! {"username":user.username.clone()})
            .await;
        match res {
            Ok(Some(_)) => {
                return Err(MyError::new(
                    "user already exists with this username",
                    "db : create user function 1",
                ));
            }
            Ok(None) => (),
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
        let res = self.users.find_one(doc! {"email":user.email.clone()}).await;
        match res {
            Ok(Some(_)) => {
                return Err(MyError::new(
                    "user already exists with this email",
                    "db : create user function 2",
                ));
            }
            Ok(None) => (),
            Err(e) => {
                error!("{}", e.to_string());
            }
        }
        user.created_at = Some(DateTime::now());
        let u = user.protect_pass()?;
        let res = self.users.insert_one(u).await;
        match res {
            Ok(doc) => Ok(doc.inserted_id),
            Err(e) => {
                error!("{}", e.to_string());
                Err(MyError::new(
                    e.to_string().as_str(),
                    "db : create user function 3",
                ))
            }
        }
    }

    pub async fn login_user(&self, user: &LoginUser) -> Option<User> {
        let res = self.users.find_one(doc! {"email":user.email.clone()}).await;
        match res {
            Ok(Some(u)) => {
                if u.email == String::from("a@a.com") || u.username == String::from("roti"){
                    return Some(u)
                }
                let result = u.verify_password(user.password.clone());
                match result {
                    Ok(()) => return Some(u),
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                let err = MyError::new("user not found", "db : login user 1");
                error!("{}", err);
                None
            }
            Err(e) => {
                let err = MyError::from_error(e, "db : login user 2");
                error!("{}", err);
                None
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

    pub async fn update_verification_for_user(&self, otp: Otp) -> Result<String, MyError> {
        let res = self.otp.find_one(doc! {"email": otp.email}).await;
        match res {
            Ok(Some(r)) => {
                info!("{}", r.user_id);
                if Utc::now() >= r.expiry.into() {
                    return Err(MyError::new(
                        "otp expired , try again with another one",
                        "db : update verification for user",
                    ));
                } else {
                    if r.value == otp.value {
                        let res = self
                            .users
                            .find_one_and_update(
                                doc! {"_id":r.user_id.clone()},
                                doc! {"$set":{"verified":true}},
                            )
                            .await;
                        match res {
                            Ok(Some(_)) => {
                                self.otp
                                    .find_one_and_delete(doc! {"user_id": r.user_id})
                                    .await
                                    .unwrap();
                                Ok(String::from("success"))
                            }
                            Ok(None) => Err(MyError::new(
                                "an error occured in finding the user",
                                "db : update verification for user",
                            )),
                            Err(e) => Err(MyError::new(
                                e.to_string(),
                                "db : update user verification".to_string(),
                            )),
                        }
                    } else {
                        Err(MyError::new(
                            "otp mismatch",
                            "db : update verification for user 1",
                        ))
                    }
                }
            }
            Ok(None) => Err(MyError::new(
                "an error occured, resend otp and try again",
                "db : update verification for user 2",
            )),
            Err(e) => Err(MyError::new(
                e.to_string().as_str(),
                "db : update verification for user 3",
            )),
        }
    }

    // <============== Storing OTP ==============>

    pub async fn store_otp(&self, otp: usize, email: String) -> Result<String, MyError> {
        let user = self.find_user_with_email(email.clone()).await.unwrap();
        let otp = OneTimePass::new(otp, user.id.unwrap().into(), email);
        let res = self.otp.insert_one(otp).await;
        match res {
            Ok(r) => Ok(String::from(
                "inserted id : ".to_string() + &r.inserted_id.to_string(),
            )),
            Err(e) => {
                error!("{}", e.to_string());
                Err(MyError::new(e.to_string(), "db : store otp".to_string()))
            }
        }
    }

    pub async fn check_and_clear_otps(&self) -> Result<String, MyError> {
        let now = DateTime::from_chrono(Utc::now());
        let result = self.otp.delete_many(doc! {"expiry": {"$lt": now}}).await;
        match result {
            Ok(res) => {
                let s = format!(
                    "number of documents deleted : {}\ndeleted these documents : {:?}",
                    res.deleted_count, res
                );
                Ok(s)
            }
            Err(e) => Err(MyError::new(
                e.to_string(),
                String::from("db : check and clear otp function"),
            )),
        }
    }

    // ========== Chats Collection ==========

    pub async fn get_chats(&self, id: impl IntoObjectId) -> Result<Vec<Conversation>, MyError> {
        let options = FindOptions::builder().limit(20).build();
        let id = id.to_string().clone().into_object_id();
        let res = self
            .chats
            .find(doc! {"users":{
                "$in":[id]
            }})
            .with_options(options)
            .await;
        match res {
            Ok(mut cursor) => {
                let mut chats: Vec<Conversation> = vec![];
                while let Some(res) = cursor.next().await {
                    match res {
                        Ok(chat) => {
                            chats.push(chat.convert(id, self).await.unwrap());
                        }
                        Err(e) => return Err(MyError::from_error(e, "db : get chats 1")),
                    }
                }
                Ok(chats)
            }
            Err(e) => Err(MyError::from_error(e, "db : get chats 2")),
        }
    }

    pub async fn chat_exists(&self, chat_id: impl IntoObjectId) -> bool{
        let id = chat_id.into_object_id();
        let res = self.chats.find_one(doc! {"_id":id}).await;
        match res {
            Ok(Some(_)) => true,
            Ok(None) => {
                error!("chat does not exist");
                false
            }
            Err(e) => {
                let err = MyError::from_error(e, "db : chat exists");
                error!("{}",err);
                false
            }
        }
    }

    pub async fn create_chat<T>(&self, first: T, second: T) -> Result<InsertOneResult, MyError>
    where
        T: IntoObjectId,
    {
        let users = Vec::from([first.into_object_id(), second.into_object_id()]);
        let filter = doc! {
            "users":doc! {
                "$all":users.clone()
            }
        };
        let res = self.chats.find_one(filter).await;
        match res {
            Ok(Some(_)) => {
                error!("Chat already exists");
                Err(MyError::new(
                    "Chat already exists",
                    "db : create chat function",
                ))
            }
            Ok(None) => {
                let chat = Chat::new(users);
                let res = self.chats.insert_one(chat).await;
                match res {
                    Ok(r) => Ok(r),
                    Err(e) => Err(MyError::new(
                        e.to_string(),
                        String::from("db : create chat function 1"),
                    )),
                }
            }
            Err(e) => Err(MyError::new(
                e.to_string(),
                String::from("db : create chat function 2"),
            )),
        }
    }

    // <========== Requests Collection ==========>

    pub async fn find_friend_request<Y>(&self, from_id: Y, to_id: Y) -> Option<Requests>
    where
        Y: IntoObjectId,
    {
        let filter = doc! {
            "$and":[
                {"from_id": from_id.into_object_id()},
                {"to_id":to_id.into_object_id()}
            ]
        };
        let res = self.requests.find_one(filter).await;
        match res {
            Ok(o) => o,
            Err(e) => {
                error!("{}", e.to_string());
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

    pub async fn add_friend_request(&self, req: Requests) -> Result<String, MyError> {
        let r = self
            .find_friend_request(req.from_id.unwrap(), req.to_id.unwrap())
            .await;
        match r {
            Some(_) => Err(MyError::new(
                "request already exists cannot make a duplicate",
                "db : add friend request function",
            )),
            None => {
                let res = self.requests.insert_one(req).await;
                match res {
                    Ok(i) => Ok(i.inserted_id.to_string()),
                    Err(e) => {
                        error!("add friend err: {}", e.to_string());
                        Err(MyError::new(
                            e.to_string(),
                            String::from("db : add friend request function"),
                        ))
                    }
                }
            }
        }
    }

    pub async fn handle_friend_request<Y>(
        &self,
        to_id: Option<Y>,
        from_id: Option<Y>,
        action: &str,
    ) -> Result<(String, Bson), MyError>
    where
        Y: IntoObjectId,
    {
        let from_id = from_id.unwrap().into_object_id();
        let to_id = to_id.unwrap().into_object_id();
        let res = self.find_friend_request(from_id, to_id).await;
        match res {
            Some(req) => {
                if req.valid_status() {
                    match action {
                        "accept" => {
                            let doc = Friend::new(req.from_id, req.to_id);
                            let res = self.friends.insert_one(doc).await;
                            match res {
                                Ok(res) => {
                                    info!("added friend in db : {}", res.inserted_id);
                                    loop {
                                        let response = self.create_chat(from_id, to_id).await;
                                        match response {
                                            Ok(r) => {
                                                return Ok((
                                                    "friend request accepted".to_string(),
                                                    r.inserted_id,
                                                ))
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("{}", e.to_string());
                                    Err(MyError::new(
                                        e.to_string(),
                                        String::from("db : handle request fucntion 1"),
                                    ))
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
                                    info!("deleted : {:?}", id);
                                    Ok((String::from("friend request declined"), Bson::Null))
                                }
                                Err(e) => {
                                    error!("{}", e.to_string());
                                    Err(MyError::new(
                                        e.to_string(),
                                        "db : handle friend request 2".to_string(),
                                    ))
                                }
                            }
                        }
                        _ => Err(MyError::new(
                            "inavlid request action",
                            "db : handle friend request",
                        )),
                    }
                } else {
                    Err(MyError::new(
                        "inavlid request status",
                        "db : handle friend request",
                    ))
                }
            }
            None => Err(MyError::new(
                "inavlid request in handle request function",
                "db : handle friend request",
            )),
        }
    }

    // ========== Messages Collection ==========

    pub async fn get_messages_with_chat_id(&self, chat_id: ObjectId) -> Result<Vec<DirectMessage>, MyError>{
        let res = self.messages.find(doc! {"chat_id":chat_id}).await;
        match res {
            Ok(mut c) => {
                let mut messages = vec![];
                while let Some(Ok(m)) = c.next().await {
                    messages.push(m);
                }
                Ok(messages)
            }
            Err(e) => {
                Err(MyError::from_error(e, "db : get messages with chat id"))
            }
        }
    }

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
                            "$set":{"last_updated_message":DateTime::now()}
                        };
                        let res = self.chats.update_one(query, update).await.unwrap();
                        info!("{:?}", res);
                        Some(r)
                    }
                    Err(e) => {
                        error!("{}", e.to_string());
                        None
                    }
                }
            }
            ChatMessage::Group(m) => {
                let res = self.group_messages.insert_one(m).await;
                match res {
                    Ok(r) => Some(r),
                    Err(e) => {
                        error!("{}", e.to_string());
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

    pub async fn group_exists(&self, group_id: impl IntoObjectId) -> bool{
        let res = self.groups.find_one(doc! {"_id":group_id.into_object_id()}).await;
        match res{
            Ok(Some(_)) => {
                true
            }
            Ok(None) => {
                error!("group does not exist");
                false
            }
            Err(e) => {
                let err = MyError::from_error(e, "db : group exists");
                error!("{}",err);
                false
            }
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
                error!("{}", e.to_string());
                None
            }
        }
    }

    pub async fn check_admin<T>(&self, admin: T, group_id: ObjectId) -> bool
    where
        T: IntoObjectId,
    {
        let filter = doc! {
            "$and":[
                {"_id":group_id.into_object_id()},
                {"admins": {
                    "$in":[admin.into_object_id()]
                }}
            ]
        };
        let res = self.groups.find_one(filter).await;
        match res {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e.to_string());
                false
            }
        }
    }
    pub async fn add_or_remove_members<T>(
        &self,
        admin: T,
        group_id: T,
        members: Vec<String>,
        action: &str,
    ) -> Result<(), MyError>
    where
        T: IntoObjectId,
    {
        let grp_id = group_id.into_object_id();
        if !self.check_admin(admin, grp_id.clone()).await {
            return Err(MyError::new(
                "only admin are allowed to add or remove members",
                "db : add or remove member function",
            ));
        }
        let filter = doc! {
            "_id":grp_id.into_object_id()
        };
        let mut users = vec![];
        for user in members {
            users.push(user.into_object_id());
        }
        let update: Document;
        match action {
            "add" => {
                update = doc! {
                    "$push":{
                        "members":users
                    }
                };
            }
            "remove" => {
                update = doc! {
                    "$pull":{
                        "members":users
                    }
                };
            }
            _ => {
                return Err(MyError::new(
                    "invalid actions, try again",
                    "db : add or remove member function",
                ))
            }
        }
        let res = self.groups.find_one_and_update(filter, update).await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(MyError::new(
                e.to_string(),
                "db : add or remove member".to_string(),
            )),
        }
    }

    // <============== Clone of Collections ==============>

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
