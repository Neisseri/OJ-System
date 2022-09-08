use std::sync::MutexGuard;
use actix_web::{get, Responder, HttpResponse};
use crate::global::{CONTEST_INFO, Contest};
use crate::persistent_storage::update_json_file;

#[get("/contests")]
async fn get_contests() -> impl Responder {
    let contest_lock: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let response: Vec<Contest> = (*contest_lock).clone();

    drop(contest_lock);
    update_json_file();
    
    HttpResponse::Ok().json(response)
}