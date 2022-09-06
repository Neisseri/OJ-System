use std::sync::MutexGuard;
use actix_web::{get, Responder, HttpResponse};
use crate::global::{CONTEST_INFO, Contest};
use crate::persistent_storage::update_json_file;

#[get("/contests")]
async fn get_contests() -> impl Responder {
    let contest_info: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let response: Vec<Contest> = (*contest_info).clone();
    drop(contest_info);
    update_json_file();
    HttpResponse::Ok().json(response)
}