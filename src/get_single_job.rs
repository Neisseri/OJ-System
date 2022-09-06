use actix_web::{get, web, Responder, HttpResponse};
use crate::global::JOB_LIST;
use crate::error::Error;
use crate::persistent_storage::update_json_file;

#[get("/jobs/{job_id}")]
async fn get_job_id(info: web::Path<usize>) -> impl Responder {

    let job_id: usize = info.into_inner();
    let lock = JOB_LIST.lock().unwrap();
    let get_jobs_json = (*lock).clone();

    if job_id > get_jobs_json.len() - 1 {
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Job 123456 not found.".to_string()
        });
    }

    let response = get_jobs_json[job_id].clone();
    let response_body = 
            serde_json::to_string_pretty(&response.clone())
            .unwrap();

    drop(lock);
    update_json_file();

    HttpResponse::Ok().body(response_body)
}