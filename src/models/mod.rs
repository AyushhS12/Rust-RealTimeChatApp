
use std::ops::{Deref};

use mongodb::bson::{oid::ObjectId , DateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone,Serialize, Deserialize,Debug)]
pub struct User{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name:String,
    pub username:String,
    pub email:String,
    password:String,
    //DateTime fields
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
    pub last_login: Option<DateTime>, // Optional
}

impl User{
    pub fn get_id(&self) -> Option<ObjectId>{
        self.id
    }
    pub fn protect_pass(&mut self) -> Self{
        self.password = String::new();
        self.deref().to_owned().clone()
    }
}
#[derive(Debug,Serialize,Deserialize)]
pub struct Friend{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id:Option<ObjectId>,
    first:Option<ObjectId>,
    second:Option<ObjectId>
}

impl Friend{
    pub fn new(first : Option<ObjectId>, second : Option<ObjectId>) -> Friend{
        Friend { id:None,first, second }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Chat{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id:Option<ObjectId>,
    users:Vec<ObjectId>,
    #[serde(default)]
    messages:Vec<Message>,
    //DateTime fields
    pub created_at: DateTime,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Group{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id:Option<ObjectId>,
    users:Vec<ObjectId>,
    #[serde(default)]
    messages:Vec<GroupMessage>,
    //DateTime fields
    pub created_at: DateTime,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Message{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id:Option<ObjectId>,
    pub chat_id:Option<ObjectId>,
    pub from_id:Option<ObjectId>,
    pub to_id:Option<ObjectId>,
    content:String,
    //DateTime fields
    pub created_at: Option<DateTime>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Requests{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id:Option<ObjectId>,
    pub from_id:Option<ObjectId>,
    pub to_id:Option<ObjectId>,
    status:String,
    //DateTime fields
    created_at: DateTime,
}


impl Requests{
    pub fn new_from_friend_req(f: FriendReq) -> Requests{
        Requests { id: None, from_id: None, to_id: f.to_id, status: String::from("pending"), created_at:DateTime::now() }
    }
    pub fn valid_status(&self) -> bool{
        self.status == "pending"
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct GroupMessage{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id:Option<ObjectId>,
    group_id:Option<ObjectId>,
    from_id:Option<ObjectId>,
    //DateTime fields
    created_at: DateTime,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct LoginUser{
    pub email:String,
    pub password:String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims{
    pub sub:String,
    pub exp:usize
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendReq{
    pub to_id:Option<ObjectId>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestHandler{
    pub from_id: Option<ObjectId>,
    pub to_id:Option<ObjectId>,
    pub action:String
}

impl RequestHandler{
    pub fn new(from_id : Option<ObjectId>, to_id: Option<ObjectId>, action:String) -> RequestHandler{
        RequestHandler { from_id, to_id, action }
    }
}