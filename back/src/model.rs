use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub enum UserState {
    CALIBRATING,
    RESTING,
    ACTIVE,
    STRESSED,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub status: String,
    pub avg_hr: f64,
    pub user_state: UserState,
    pub timestamp: u64,
    pub hr_data: Vec<StoredInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredInfo {
    pub timestamp: u64,
    pub hr: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub key: String,
    pub hr: f64,
    pub timestamp: u64,
}
