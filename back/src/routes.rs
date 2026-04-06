use once_cell::sync::Lazy;
use rocket::{http::Status, serde::json::Json};
use std::{collections::VecDeque, sync::Mutex};

use crate::model::{Info, Key, StoredInfo};

static KEY: &'static str = "somereallysecurecryptographickeyofsomesort";

static CALIBRATION_PERIOD_SECONDS: usize = 10;

static USER_STATUSES: Lazy<Mutex<VecDeque<StoredInfo>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(60)));

/// Flushes the user statuses.
///
/// ### Sample Request
///
/// ```python
/// import requests
///
/// requests.post(
///     "https://example.com/api/flush",
///     json={
///         "key": "somereallysecurecryptographickeyofsomesort",
///     },
/// )
/// ```
///
/// ### Response Info
///
/// HTTP 200 if the user statuses were flushed successfully.
/// HTTP 401 if the key is invalid.
/// HTTP 500 if the data could not be inserted.
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

/// Insert user heartrate data to the in-memory database.
///
/// ### Sample Request
///
/// ```python
/// import requests
///
/// requests.post(
///     "https://example.com/api/info",
///     json={
///         "key": "somereallysecurecryptographickeyofsomesort",
///         "hr": 60.0,
///         "timestamp": 1234567890,
///     },
/// )
/// ```
///
/// ### Response Info
///
/// HTTP 200 if the data was inserted correctly.
/// HTTP 401 if the key is invalid.
/// HTTP 500 if the data could not be inserted.
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

            if statuses.len() > 60 {
                statuses.pop_front();
            } else if statuses.len() == CALIBRATION_PERIOD_SECONDS {
                // ... do some logic to calculate resting heart rate and variance
            }
        }
        Err(e) => {
            log::error!("Failed to lock USER_STATUSES: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    Ok(Status::Ok)
}

/// Performs some mathematics in the background using the stored heartrate data,
/// and returns the result of the mathematics, indicating whether the current
/// heart rate is nominal or elevated.
///
/// ### Sample Request
///
/// ```python
/// import requests
///
/// requests.get(
///     "https://example.com/api/info?key=somereallysecurecryptographickeyofsomesort",
/// )
/// ```
///
/// ### Sample Response
///
/// ```json
/// {
///   "status": "NOMINAL",
///   "avg_hr": 60.0,
///   "user_state": "NOMINAL",
///   "timestamp": 1234567890,
///   "hr_data": [
///     {
///       "timestamp": 1234567890,
///       "hr": 60.0
///     },
///     {
///       "timestamp": 1234567891,
///       "hr": 60.0
///     }
///   ]
/// }
/// ```
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
