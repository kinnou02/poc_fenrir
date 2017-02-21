#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
extern crate serde;
extern crate mongo_driver;
extern crate chrono;
extern crate rocket_contrib;
#[macro_use(bson, doc)] extern crate bson;
#[macro_use] extern crate serde_derive;

use rocket_contrib::JSON;

use bson::{EncoderError, DecoderError};

use std::sync::Arc;
use mongo_driver::client::{ClientPool,Uri};

use rocket::State;

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    status: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub coverage: Option<String>,
    pub navitia_token: Option<String>,
    pub contributor_code: Option<String>,
}


#[get("/status")]
fn status() -> JSON<StatusResponse> {
    JSON(StatusResponse{status: String::from("ok")})
}


#[post("/users", data = "<user>")]
fn add_user(pool: State<Arc<ClientPool>>, user: JSON<User>) -> Result<JSON<User>, EncoderError> {
    let client =  pool.pop();
    let user = user.into_inner();
    let db = client.get_database("fenrir");
    let coll = db.get_collection("users");
    let b = bson::to_bson(&user)?;
    if let bson::Bson::Document(document) = b {
        coll.insert(&document, None).unwrap();
    }
    Ok(JSON(user))
}

#[get("/users/<id>")]
fn get_user(pool: State<Arc<ClientPool>>, id: &str) -> Result<JSON<User>, DecoderError> {
    let client =  pool.pop();
    let db = client.get_database("fenrir");
    let coll = db.get_collection("users");
    let mut item = coll.find(&doc!{"_id" => id}, None).ok().expect("Document not found");
    let user = bson::from_bson(bson::Bson::Document(item.next().unwrap().unwrap()))?;
    Ok(JSON(user))
}

#[get("/users")]
fn get_users(pool: State<Arc<ClientPool>>) -> Result<JSON<Vec<User>>, DecoderError> {
    let client =  pool.pop();
    let db = client.get_database("fenrir");
    let coll = db.get_collection("users");
    let iter = coll.find(&doc!{}, None).ok().expect("Document not found");
    let users = iter.filter_map(|doc| {
        if let Ok(item) = doc{
            bson::from_bson(bson::Bson::Document(item)).ok()
        }else{
            None
        }
    });
    Ok(JSON(users.collect()))
}


fn main() {
    let uri = Uri::new("mongodb://localhost:27017/").unwrap();
    let pool = Arc::new(ClientPool::new(uri.clone(), None));
    rocket::ignite().manage(pool).mount("/", routes![status, add_user, get_user, get_users]).launch();
}

