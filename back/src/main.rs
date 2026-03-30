#[macro_use]
extern crate rocket;

use once_cell::sync::Lazy;
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
};
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
struct Info {
    key: String,
    hr: f64,
    timestamp: u64,
}

static KEY: &'static str = "somereallysecurecryptographickeyofsomesort";

#[derive(Debug, Clone)]
struct StoredInfo {
    timestamp: u64,
    hr: f64,
}

#[derive(Debug, Clone, Serialize)]
struct Response {
    status: String,
    hr: f64,
    timestamp: u64,
}

static USER_STATUSES: Lazy<Mutex<Vec<StoredInfo>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[post("/info", format = "json", data = "<info>")]
async fn post_info(info: Json<Info>) -> Result<Status, Status> {
    if info.key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(mut statuses) => {
            statuses.push(StoredInfo {
                timestamp: info.timestamp,
                hr: info.hr,
            });
        }
        Err(_) => {
            return Err(Status::InternalServerError);
        }
    };

    Ok(Status::Ok)
}

#[get("/info")]
async fn get_info() -> Result<Status, Status> {
    Ok(Status::Ok)
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![post_info, get_info])
}
