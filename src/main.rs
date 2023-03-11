#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use rocket::config::{Config, Environment};
use rocket::http::{Method, Status};
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use std::path::{PathBuf};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::net::{TcpListener, TcpStream};
use hostname::get;

fn get_ip_addr() -> String {
    let orig_addr = SocketAddr::new(
        IpAddr::V4(
            Ipv4Addr::new(0, 0, 0, 0)
        ),
        0
        );
    let listener = TcpListener::bind(&orig_addr).unwrap();
    let addr = listener.local_addr().unwrap();
    let ip = match addr.ip() {
        IpAddr::V4(ipv4) => ipv4.to_string(),
        IpAddr::V6(ipv6) => ipv6.to_string(),
    };
    ip
}

fn rocket() -> rocket::Rocket {
    // let allowed_origins = AllowedOrigins::some_exact(&[ // 4.
    //     //CHANGE THESE TO MATCH YOUR PORTS
    //     "http://localhost:3000",
    //     "http://127.0.0.1:3000",
    //     "http://localhost:8000",
    //     "http://0.0.0.0:8000",
    //     "http://192.168.0.11:3000"]);
    let ip_addr = get_ip_addr();
    let allowed_origins = AllowedOrigins::all();
        
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![
            Method::Get, 
            Method::Post,
            Method::Options,
            Method::Put,
            Method::Delete,
            Method::Patch,
            Method::Head,
            ].into_iter().map(From::from).collect(), 
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS fairing");

    let config = Config::build(Environment::Production)
        .address(ip_addr)
        .port(8000)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .mount("/", routes![get_items, add_item])
        .attach(cors)
        .mount("/", routes![catch_options])
}

#[options("/<_path..>")]
fn catch_options(_path: PathBuf) -> rocket::Response<'static> {
    let status = rocket::http::Status::Ok;
    let mut response = rocket::Response::new();
    response.set_status(status);
    response.set_raw_header("Access-Control-Allow-Origin", "*");
    response.set_raw_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
    response.set_raw_header("Access-Control-Allow-Headers", "Content-Type");

    response
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
    let ip_addr = get_ip_addr();
    println!("This is the ip address: {}",ip_addr);
    rocket().launch();
}
