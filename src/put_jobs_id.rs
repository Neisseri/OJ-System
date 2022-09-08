use std::sync::MutexGuard;

use actix_web::{put, web, Responder, HttpResponse};
use crate::{global::JOB_LIST, response::Response};
use crate::error::Error;
use chrono::{Utc, SecondsFormat};
use crate::persistent_storage::update_json_file;

#[put("/jobs/{job_id}")]
async fn put_jobs(path: web::Path<usize>) -> impl Responder {
    
    let job_id: usize = path.into_inner();
    let mut lock: MutexGuard<Vec<Response>> = JOB_LIST.lock().unwrap();
    let job_list: Vec<Response> = (*lock).clone();

    if job_id as i32 > job_list.len() as i32 - 1 {
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Job 123456 not found.".to_string()
        });
    }

    (*lock)[job_id].updated_time = Utc::now().
        to_rfc3339_opts(SecondsFormat::Millis, true);
    let response: Response = (*lock)[job_id].clone();

    drop(lock);
    update_json_file();

    HttpResponse::Ok().json(response)
}