use std::sync::MutexGuard;

use actix_web::{get, web, Responder, HttpResponse};
use crate::global::JOB_LIST;
use crate::error::Error;
use crate::persistent_storage::update_json_file;
use crate::response::Response;

#[get("/jobs/{job_id}")]
async fn get_job_id(info: web::Path<usize>) -> impl Responder {

    let job_id: usize = info.into_inner();
    let job_lock: MutexGuard<Vec<Response>> = JOB_LIST.lock().unwrap();
    let job_list: Vec<Response> = (*job_lock).clone();

    if job_id > job_list.len() - 1 { // The job_id is INVALID
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: format!("Job {} not found.", job_id).clone(),
        }); // 404 Not Found
    }
    // job_id is from 0  to job_list.len() - 1

    let response: Response = job_list[job_id].clone();

    drop(job_lock);
    update_json_file();

    HttpResponse::Ok().json(response)
}