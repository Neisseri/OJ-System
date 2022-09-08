use std::sync::MutexGuard;
use std::io::Error;
use serde::{Serialize, Deserialize};
use crate::global::{Submit, User, Contest, 
    GLOBAL_CONTEST_LIST, USER_LIST, CONTEST_INFO, JOB_LIST};
use crate::response::Response;

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistentStorage {
    pub judge_task: Option<Vec<Submit>>,
    pub user_info: Option<Vec<User>>,
    pub contest_info: Option<Vec<Contest>>,
    pub job_list: Option<Vec<Response>>,
}

pub fn clear_persistent_storage() {
    let address: String = "./persistent_storage.json".to_string();
    std::fs::write(&address, "{}")
        .expect("Clear Storage Error");
}

pub fn read_persistent_storage() {
    let mut submit_lock: MutexGuard<Vec<Submit>> = 
        GLOBAL_CONTEST_LIST.lock().unwrap();
    let mut user_lock: MutexGuard<Vec<User>> = USER_LIST.lock().unwrap();
    let mut contest_lock: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let mut job_lock: MutexGuard<Vec<Response>> = JOB_LIST.lock().unwrap();
    let address: String = "./persistent_storage.json".to_string();

    let storage_json: PersistentStorage = {
        let json_record: Result<String, Error> = std::fs::read_to_string(&address);
        let s: String = json_record.unwrap();
        serde_json::from_str::<PersistentStorage>(&s).unwrap()
    };
    if storage_json.judge_task.is_some() == true {
        (*submit_lock) = storage_json.judge_task.clone().unwrap();
    }
    if storage_json.user_info.is_some() == true {
        (*user_lock) = storage_json.user_info.clone().unwrap();
    }
    if storage_json.contest_info.is_some() == true {
        (*contest_lock) = storage_json.contest_info.clone().unwrap();
    }
    if storage_json.job_list.is_some() == true {
        (*job_lock) = storage_json.job_list.clone().unwrap();
    }
}

pub fn update_json_file() {
    let submit_lock: MutexGuard<Vec<Submit>> = GLOBAL_CONTEST_LIST.lock().unwrap();
    let user_lock: MutexGuard<Vec<User>> = USER_LIST.lock().unwrap();
    let contest_lock: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let job_lock: MutexGuard<Vec<Response>> = JOB_LIST.lock().unwrap();
    let address: String = "./persistent_storage.json".to_string();
    
    let response: PersistentStorage = PersistentStorage { 
        judge_task: Some((*submit_lock).clone()), 
        user_info: Some((*user_lock).clone()), 
        contest_info: Some((*contest_lock).clone()), 
        job_list: Some((*job_lock).clone()),
    };
    std::fs::write(
        &address,
        serde_json::to_string_pretty(&response).unwrap(),
    ).expect("WRITE IN ERROR!"); // write in the json file
}