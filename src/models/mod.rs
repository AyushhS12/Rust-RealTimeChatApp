
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug)]
pub struct User{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name:String,
    email:String,
    password:String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    received_reqs: Vec<ObjectId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    sent_reqs: Vec<ObjectId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    chats:Vec<ObjectId>
}

impl User{
    pub fn get_id(&self) -> Option<ObjectId>{
        self.id
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Chat{
    #[serde(rename = "_id")]
    id:Option<ObjectId>,
    users:Vec<ObjectId>,
    #[serde(default)]
    messages:Vec<Message>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Group{
    #[serde(rename = "_id")]
    id:Option<ObjectId>,
    users:Vec<ObjectId>,
    #[serde(default)]
    messages:Vec<GroupMessage>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Message{
    #[serde(rename = "_id")]
    id:Option<ObjectId>,
    chat_id:Option<ObjectId>,
    from_id:Option<ObjectId>,
    to_id:Option<ObjectId>,
    content:String
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Requests{
    #[serde(rename = "_id")]
    id:Option<ObjectId>,
    from_id:Option<ObjectId>,
    to_id:Option<ObjectId>,
    status:String
}

#[derive(Debug,Serialize,Deserialize)]
pub struct GroupMessage{
    #[serde(rename = "_id")]
    id:Option<ObjectId>,
    group_id:Option<ObjectId>,
    from_id:Option<ObjectId>
}

#[derive(Serialize, Deserialize,Debug)]
pub struct LoginUser{
    pub email:String,
    pub password:String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims{
    pub sub:User,
    pub exp:Option<usize>
}