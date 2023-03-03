#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use rocket::http::{Method, Status};
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use std::path::{PathBuf};
use std::net::{IpAddr, Ipv4Addr};

fn rocket() -> rocket::Rocket {
    let allowed_origins = AllowedOrigins::all();
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Options]
                            .into_iter()
                            .map(From::from)
                            .collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::ignite()
        .mount("/", routes![get_items, add_item])
        .attach(cors)
        .mount("/", routes![catch_options])
}

#[options("/<_path..>")]
fn catch_options(_path: PathBuf) -> rocket::http::Status {
    rocket::http::Status::Ok
}
// A struct to represent a to-do item. We'll use this to send and receive
#[derive(Debug, Clone, Serialize, Deserialize,FromForm)]
struct TodoItem {
    id: Option<i32>,
    title: String,
    completed: bool,
}

// A global counter for generating unique IDs for new items
static mut COUNTER: i32 = 0;

// A Mutex-protected Vec to store the items
static ITEMS: Mutex<Vec<TodoItem>> = Mutex::new(Vec::new());

// Endpoint to get all items
#[get("/items")]
fn get_items() -> Json<Vec<TodoItem>> {
    let items = ITEMS.lock().unwrap();
    Json(items.clone())
}

// Endpoint to add a new item
#[post("/items", format = "json", data = "<item>")]
fn add_item(item: Json<TodoItem>) -> Result<Json<Vec<TodoItem>>, Status> {
    let mut items = ITEMS.lock().unwrap();
    let new_id = match items.last() {
        Some(last_item) => last_item.id.unwrap() + 1,
        None => 1,
    };
    let mut new_item = item.into_inner();
    new_item.id = Some(new_id);
    items.push(new_item);
    Ok(Json(items.clone()))
}

fn main() {
    rocket().launch();
}
