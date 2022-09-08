use std::sync::MutexGuard;
use actix_web::{get, Responder, HttpResponse, web};
use crate::global::{CONTEST_INFO, Contest};
use crate::error::Error;
use crate::persistent_storage::update_json_file;

#[get("/contests/{contest_id}")]
async fn get_contests_id(path: web::Path<usize>) -> impl Responder {

    let contest_id: usize = path.into_inner();
    // get the contest_id from path
    let contest_lock: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let contest_info: Vec<Contest> = (*contest_lock).clone();
    // the information of all the contests
    let contest_num: usize = contest_info.len();
    // get the global variables for contest information

    // contest ids are from 1 to contest_num
    if contest_id > contest_num {
        return HttpResponse::NotFound().json(Error { // 404 Not Found
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Contest 114514 not found.".to_string(),
        });
    } // can't find the contest id
    // contest_id -> contest_info[contest_id - 1]
    let response: Contest = contest_info[contest_id - 1].clone();

    drop(contest_lock);
    update_json_file();

    HttpResponse::Ok().json(response)
}