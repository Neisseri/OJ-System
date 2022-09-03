use actix_web::{put, web, Responder, HttpResponse, http};
use crate::global::{JOB_LIST, JOB_NUM, self};
use crate::config::Config;
use crate::response::{Response, CaseResult, Result, State};
use crate::error::Error;
use std::fs;
use std::io::Write;    
use std::time::{Duration, Instant};
use std::process::{Command, Stdio};
use wait_timeout::ChildExt;
use chrono::{Utc, SecondsFormat};
use crate::tool::vec_char_equal;

#[put("/jobs/{job_id}")]
async fn put_jobs(path: web::Path<usize>,
        config: web::Data<Config>) -> impl Responder {
    
    let job_id: usize = path.into_inner();
    let mut lock = JOB_LIST.lock().unwrap();
    let mut job_list = (*lock).clone();

    if job_id as i32 > job_list.len() as i32 - 1 {
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Job 123456 not found.".to_string()
        });
    }

    (*lock)[job_id].updated_time = Utc::now().
        to_rfc3339_opts(SecondsFormat::Millis, true);
    let response = (*lock)[job_id].clone();

    let response_body = 
        serde_json::to_string_pretty(&response.clone())
        .unwrap();
    // change the struct to json format String

    HttpResponse::Ok().body(response_body)
}