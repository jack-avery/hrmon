use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub enum UserState {
    /// The user's resting heart rate is calibrating
    CALIBRATING,

    /// The user's heart rate is nominal
    NOMINAL,

    /// The user's heart rate is elevated
    ELEVATED,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    /// The current state of the user
    pub user_state: UserState,

    /// The average heartrate of the user
    pub avg_hr: f64,

    /// The heartrate data of the user
    pub hr_data: Vec<StoredInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredInfo {
    /// Epoch second for this measurement
    pub timestamp: u64,

    /// Average heartrate for this epoch second, measured by the client
    pub hr: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    /// User key
    pub key: String,

    /// Epoch second for this measurement
    pub timestamp: u64,

    /// Average heartrate for this epoch second, measured by the client
    pub hr: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Key {
    /// User key
    pub key: String,
}
