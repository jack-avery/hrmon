use once_cell::sync::Lazy;
use rocket::{http::Status, serde::json::Json};
use std::{collections::VecDeque, sync::Mutex};

use crate::model::{Info, Key, StoredInfo};

static KEY: &'static str = "somereallysecurecryptographickeyofsomesort";

static USER_STATUSES: Lazy<Mutex<VecDeque<StoredInfo>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(60)));

#[post("/flush", format = "json", data = "<key>")]
pub async fn flush_info(key: Json<Key>) -> Result<Status, Status> {
    if key.key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(mut statuses) => {
            statuses.clear();
        }
        Err(e) => {
            log::error!("Failed to lock USER_STATUSES: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    Ok(Status::Ok)
}

#[post("/info", format = "json", data = "<info>")]
pub async fn post_info(info: Json<Info>) -> Result<Status, Status> {
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
        Err(e) => {
            log::error!("Failed to lock USER_STATUSES: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    Ok(Status::Ok)
}

#[get("/info?<key>")]
pub async fn get_info(key: String) -> Result<Status, Status> {
    if key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            log::error!("Failed to lock USER_STATUSES: {}", e);
            return Err(Status::InternalServerError);
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![flush_info, post_info, get_info]
}
