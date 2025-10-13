use std::{collections::HashSet};

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use log::error;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::db::{Db, IntoObjectId};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub email: String,
    password: String,
    //Verification
    #[serde(skip_deserializing, default)]
    pub verified: bool,
    //DateTime fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub last_login: Option<DateTime>,
}

impl User {
    pub fn protect_pass(&mut self) -> Result<User, MyError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password = argon2.hash_password(self.password.as_bytes(), &salt);
        match password {
            Ok(pass) => {
                self.password = pass.to_string();
                Ok(self.to_owned())
            }
            Err(e) => {
                Err(MyError::from_error(e, "models : protect pass"))
            }
        }
    }

    pub fn verify_password(&self,password:String) -> Result<(), MyError> {
        let argon2 = Argon2::default();
        let hash = PasswordHash::new(&self.password).unwrap();
        let res = argon2.verify_password(password.as_bytes(), &hash);
        match res {
            Ok(()) => {
                Ok(())
            }
            Err(e) => {
                Err(MyError::from_error(e, "models : verify password"))
            }
        }
    }

    pub fn hide_pass(&mut self) -> User {
        self.password = "".to_string();
        self.to_owned()
    }
}

// #[derive(Clone, Serialize, Deserialize, Debug)]
// pub struct OneTimePass {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     id: Option<ObjectId>,
//     pub value: usize,
//     pub expiry: DateTime,
//     pub email: String,
//     pub user_id: Bson,
// }

// impl OneTimePass {
//     pub fn new(value: usize, id: Bson, email: String) -> OneTimePass {
//         let expiry = (Utc::now() + Duration::minutes(10)).into();
//         OneTimePass {
//             id: None,
//             value,
//             expiry,
//             user_id: id,
//             email,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Friend {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    users: [Option<ObjectId>; 2],
    //Time
    created_at: DateTime,
}

impl Friend {
    pub fn new(first: Option<ObjectId>, second: Option<ObjectId>) -> Friend {
        Friend {
            id: None,
            users: [first, second],
            created_at: DateTime::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    users: Vec<ObjectId>,
    //DateTime fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_update: Option<DirectMessage>,
    pub created_at: DateTime,
}

impl Chat {
    pub fn new(users: Vec<ObjectId>) -> Chat {
        let msg = DirectMessage {
            id: None,
            to_id: None,
            from_id: None,
            content: String::new(),
            chat_id: None,
            created_at: Some(DateTime::now()),
        };
        Chat {
            id: None,
            users,
            created_at: DateTime::now(),
            last_message_update: Some(msg),
        }
    }

    pub async  fn convert(&self, id: impl IntoObjectId,db: &Db) -> Option<Conversation> {
        match self.users.as_slice() {
            [user1, user2] => {
                if user1.to_hex() == id.to_string() {
                    let user = db.find_user_with_id(user2.into_object_id()).await.unwrap();
                    Some(Conversation {
                        id:self.id,
                        sender: *user1,
                        receiver: TempUser { id: user.id.unwrap(), name: user.name, username: user.username },
                        last_updated_message:self.last_message_update.clone()
                    })
                } else {
                    let user = db.find_user_with_id(user1.into_object_id()).await.unwrap();
                    Some(Conversation {
                        id:self.id,
                        sender: *user2,
                        receiver: TempUser { id: user.id.unwrap(), name:user.name, username: user.username},
                        last_updated_message:self.last_message_update.clone()
                    })
                }
            }
            _ => {
                error!("cannot convert because chat have more than two members");
                None
            },
        }
    }
}

#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct Conversation{
    id:Option<ObjectId>,
    sender: ObjectId,
    receiver: TempUser,
    last_updated_message:Option<DirectMessage>
}
#[derive(Deserialize,Serialize,Debug,Clone)]
struct TempUser{
    id:ObjectId,
    name:String,
    username:String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    admins: HashSet<ObjectId>,
    pub members: HashSet<ObjectId>,
    //DateTime fields
    pub created_at: DateTime,
}

impl Group {
    pub fn new(admins: HashSet<ObjectId>, members: HashSet<ObjectId>) -> Group {
        Group {
            id: None,
            admins,
            members,
            created_at: DateTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub chat_id: Option<ObjectId>,
    pub from_id: Option<ObjectId>,
    pub to_id: Option<ObjectId>,
    content: String,
    //DateTime fields
    pub created_at: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Requests {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    pub from_id: Option<ObjectId>,
    pub to_id: Option<ObjectId>,
    status: String,
    //DateTime fields
    created_at: DateTime,
}

impl Requests {
    pub fn new_from_friend_req(f: FriendReq, from_id: Option<ObjectId>) -> Requests {
        Requests {
            id: None,
            from_id,
            to_id: f.to_id,
            status: String::from("pending"),
            created_at: DateTime::now(),
        }
    }
    pub fn valid_status(&self) -> bool {
        self.status == "pending"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    pub group_id: Option<ObjectId>,
    pub from_id: Option<ObjectId>,
    //DateTime fields
    created_at: DateTime,
}

//Utility Models

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendReq {
    pub to_id: Option<ObjectId>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct RequestHandler{
//     pub from_id: Option<ObjectId>,
//     pub to_id:Option<ObjectId>
// }

// impl RequestHandler{
//     pub fn new(from_id : Option<ObjectId>, to_id: Option<ObjectId>) -> RequestHandler{
//         RequestHandler { from_id, to_id}
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "lowercase")]
pub enum FriendRequest {
    Accept { from_id: String },
    Reject { from_id: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Members {
    pub members: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub second: Option<ObjectId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ChatMessage {
    Direct(DirectMessage),
    Group(GroupMessage),
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TempDirect{
//     pub to_id:Option<ObjectId>,
//     pub chat_id:Option<ObjectId>,
//     pub content:String
// }

// Error Handling Shenanigans
#[derive(Debug)]
pub struct MyError {
    error: String,
    location: String,
}
impl MyError {
    pub fn new<T: ToString>(error: T, location: T) -> MyError {
        MyError {
            error: error.to_string(),
            location: location.to_string(),
        }
    }
    #[allow(dead_code)]
    pub fn into_error(self) -> String {
        self.error
    }
    pub fn error(&self) -> &str {
        &self.error
    }
    pub fn from_error<T>(e: T, location: &str) -> MyError 
    where T : ToString + ToOwned{
        MyError {
            error: e.to_string(),
            location: location.to_string(),
        }
    }
}
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[error : {} , location: {}]", self.error, self.location)
    }
}
impl std::error::Error for MyError {}


impl From<mongodb::error::Error> for MyError{
    fn from(value: mongodb::error::Error) -> Self {
        MyError { error: value.to_string(), location: "don't know".to_string() }
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Profile {
//     user: User,
//     friends: Friend,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Otp {
//     pub email: String,
//     pub value: usize,
// }
