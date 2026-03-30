#[macro_use]
extern crate rocket;

use once_cell::sync::Lazy;
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
};
use std::{collections::VecDeque, sync::Mutex};

#[derive(Debug, Clone, Deserialize)]
struct Info {
    key: String,
    hr: f64,
    timestamp: u64,
}

static KEY: &'static str = "somereallysecurecryptographickeyofsomesort";

#[derive(Debug, Clone, Serialize)]
struct StoredInfo {
    timestamp: u64,
    hr: f64,
}

#[derive(Debug, Clone, Serialize)]
enum UserState {
    CALIBRATING,
    RESTING,
    ACTIVE,
    STRESSED,
}

#[derive(Debug, Clone, Serialize)]
struct Response {
    status: String,
    avg_hr: f64,
    user_state: UserState,
    timestamp: u64,
    hr_data: Vec<StoredInfo>,
}

static USER_STATUSES: Lazy<Mutex<VecDeque<StoredInfo>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(60)));

#[post("/info", format = "json", data = "<info>")]
async fn post_info(info: Json<Info>) -> Result<Status, Status> {
    if info.key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(mut statuses) => {
            statuses.push_back(StoredInfo {
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

#[get("/info?<key>")]
async fn get_info(key: String) -> Result<Status, Status> {
    if key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(_) => Ok(Status::Ok),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![post_info, get_info])
}
