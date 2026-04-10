use once_cell::sync::Lazy;
use rocket::{http::Status, serde::json::Json};
use std::{collections::VecDeque, sync::Mutex};

use crate::model::{Info, InfoResponse, Key, StoredInfo, UserState};

static KEY: &'static str = "somereallysecurecryptographickeyofsomesort";

static CALIBRATION_PERIOD_SECONDS: usize = 10;
static STD_DEVS_TO_CONSIDER_ELEVATED: f64 = 2.0;

static USER_STATUSES: Lazy<Mutex<VecDeque<StoredInfo>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(60)));

static AVG_HR: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.0));
static STD_DEV_HR: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.0));

/// Flushes state.
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
/// HTTP 200 if state was flushed successfully.
/// HTTP 401 if the key is invalid.
/// HTTP 500 if a portion of state could not be flushed.
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

    match AVG_HR.lock() {
        Ok(mut avg_hr) => {
            *avg_hr = 0.0;
        }
        Err(e) => {
            log::error!("Failed to lock AVG_HR: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    match STD_DEV_HR.lock() {
        Ok(mut std_dev_hr) => {
            *std_dev_hr = 0.0;
        }
        Err(e) => {
            log::error!("Failed to lock STD_DEV_HR: {}", e);
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
                match AVG_HR.lock() {
                    Ok(mut avg_hr) => {
                        *avg_hr =
                            statuses.iter().map(|s| s.hr).sum::<f64>() / statuses.len() as f64;

                        match STD_DEV_HR.lock() {
                            Ok(mut std_dev_hr) => {
                                *std_dev_hr = statuses
                                    .iter()
                                    .map(|s| (s.hr - *avg_hr).powi(2))
                                    .sum::<f64>()
                                    / statuses.len() as f64;
                            }
                            Err(e) => {
                                log::error!("Failed to lock STD_DEV_HR: {}", e);
                                return Err(Status::InternalServerError);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to lock AVG_HR: {}", e);
                        return Err(Status::InternalServerError);
                    }
                }
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
pub async fn get_info(key: String) -> Result<Json<InfoResponse>, Status> {
    if key != KEY {
        return Err(Status::Unauthorized);
    }

    match USER_STATUSES.lock() {
        Ok(statuses) => {
            if statuses.len() < CALIBRATION_PERIOD_SECONDS {
                return Ok(Json(InfoResponse {
                    user_state: UserState::CALIBRATING,
                    avg_hr: 0.0,
                    hr_data: statuses.iter().map(|s| s.clone()).collect(),
                }));
            }

            let avg_hr = AVG_HR.lock().unwrap();
            let std_dev_hr = STD_DEV_HR.lock().unwrap();

            let user_state = if *avg_hr + *std_dev_hr * STD_DEVS_TO_CONSIDER_ELEVATED
                < statuses.back().unwrap().hr
            {
                UserState::ELEVATED
            } else {
                UserState::NOMINAL
            };

            return Ok(Json(InfoResponse {
                user_state,
                avg_hr: *avg_hr,
                hr_data: statuses.iter().map(|s| s.clone()).collect(),
            }));
        }
        Err(e) => {
            log::error!("Failed to lock USER_STATUSES: {}", e);
            return Err(Status::InternalServerError);
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![flush_info, post_info, get_info]
}
