#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
extern crate serde;
extern crate mongodb;
#[macro_use(bson, doc)]
extern crate bson;

#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate chrono;
#[macro_use] extern crate log;

use rocket_contrib::JSON;

use bson::{Bson, Document, EncoderError, DecoderError};
use mongodb::{Client, ThreadedClient};
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

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
}

type MongoClient = std::sync::Arc<mongodb::ClientInner>;

#[get("/status")]
fn status() -> JSON<StatusResponse> {
    JSON(StatusResponse{status: String::from("ok")})
}


#[post("/users", data = "<user>")]
fn add_user(client: State<MongoClient>, user: JSON<User>) -> Result<JSON<User>, EncoderError> {
    let user = user.into_inner();
    let coll = client.db("fenrir").collection("users");
    let b = bson::to_bson(&user)?;
    if let bson::Bson::Document(document) = b {
        coll.insert_one(document, None).unwrap();
    }
    Ok(JSON(user))
}

#[get("/users/<id>")]
fn get_user(client: State<MongoClient>, id: &str) -> Result<JSON<User>, DecoderError> {
    info!("start get db: {:?}", Local::now());
    let coll = client.db("fenrir").collection("users");
    info!("finish get db: {:?}", Local::now());
    let item = coll.find_one(Some(doc!{"_id" => id}), None).ok().expect("Document not found");
    info!("finish request: {:?}", Local::now());
    let user = bson::from_bson(bson::Bson::Document(item.unwrap()))?;
    info!("to bson: {:?}", Local::now());
    Ok(JSON(user))
}


fn main() {
    let client = Client::with_uri("mongodb://localhost:27017")
                         .expect("Failed to initialize client.");
    rocket::ignite().manage(client).mount("/", routes![status, add_user, get_user]).launch();
}

