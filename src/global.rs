use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::response::Response;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: usize,
    pub name: String,
}

lazy_static! {
    pub static ref JOB_NUM: Arc<Mutex<u64>> = 
        Arc::new(Mutex::new(0));
} // record the serial number of judge jobs

lazy_static! {
    pub static ref JOB_LIST: Arc<Mutex<Vec<Response>>>
        = Arc::new(Mutex::new(Vec::new()));
}

lazy_static! {
    pub static ref USER_LIST: Arc<Mutex<Vec<User>>>
         = Arc::new(Mutex::new(vec![User {
            id: 0,
            name: "root".to_string(),
         }]));
}