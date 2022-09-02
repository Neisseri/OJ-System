use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Error {
    pub code: u64,
    pub reason: String,
    pub message: String
}