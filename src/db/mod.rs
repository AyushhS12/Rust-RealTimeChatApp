use crate::models::*;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson, Document},
    Client, Collection,
};
use std::{env, str::FromStr, sync::Arc};

trait IntoObjectId {
    fn into_object_id(self) -> ObjectId;
}

#[derive(Clone)]
pub struct Db {
    users: Arc<Collection<User>>,
    chats: Arc<Collection<Chat>>,
    messages: Arc<Collection<Message>>,
    groups: Arc<Collection<Group>>,
    requests: Arc<Collection<Requests>>,
    group_messages: Arc<Collection<GroupMessage>>,
}

impl IntoObjectId for String{
    fn into_object_id(self) -> ObjectId {
        ObjectId::from_str(&self).unwrap()   
    }
}

impl IntoObjectId for ObjectId{
    fn into_object_id(self) -> ObjectId {
        self
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
                let chats = Arc::new(db.collection::<Chat>("chats"));
                let messages = Arc::new(db.collection::<Message>("messages"));
                let groups = Arc::new(db.collection::<Group>("groups"));
                let requests = Arc::new(db.collection::<Requests>("requests"));
                let group_messages = Arc::new(db.collection::<GroupMessage>("group_messages"));
                Ok(Db {
                    users,
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
            Ok(r)=>{
                r
            },
            Err(e)=>{
                println!("{}",e.to_string());
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

    pub async fn create_user(&self, user: &User) -> Result<Bson, String> {
        let res = self.users.insert_one(user).await;
        match res {
            Ok(doc) => Ok(doc.inserted_id),
            Err(e) => {
                println!("{}", e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn users(self) -> Arc<Collection<User>> {
        self.users.clone()
    }
    pub async fn chats(self) -> Arc<Collection<Chat>> {
        self.chats.clone()
    }
    pub async fn messages(self) -> Arc<Collection<Message>> {
        self.messages.clone()
    }
    pub async fn requests(self) -> Arc<Collection<Requests>> {
        self.requests.clone()
    }
    pub async fn groups(self) -> Arc<Collection<Group>> {
        self.groups.clone()
    }
    pub async fn groups_messages(self) -> Arc<Collection<GroupMessage>> {
        self.group_messages.clone()
    }
}
