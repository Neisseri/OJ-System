use std::sync::MutexGuard;

use actix_web::{get, web, Responder, HttpResponse};
use crate::response::{State, Result, Response};
use serde::{Serialize, Deserialize};
use crate::global::JOB_LIST;
use chrono::NaiveDateTime;
use crate::persistent_storage::update_json_file;

#[derive(Serialize, Deserialize)]
struct ListFilter {
    user_id: Option<usize>,
    user_name: Option<String>,
    contest_id: Option<usize>,
    problem_id: Option<usize>,
    language: Option<String>,
    from: Option<String>, // Start Time
    to: Option<String>, // End Time
    state: Option<State>,
    result: Option<Result>,
} // the arguments for filtering jobs

#[get("/jobs")]
async fn get_jobs(info: web::Query<ListFilter>) -> impl Responder {

    let job_lock: MutexGuard<Vec<Response>> = JOB_LIST.lock().unwrap();
    let job_list: Vec<Response> = (*job_lock).clone();
    let mut response: Vec<Response> = Vec::new();
    // the jobs after filtered

    // obey the ascending order for times
    let mut job_index: i32 = 0; // use i32, as if use `usize`, 
    // it can't be minused 1 to `-1`
    while job_index <= job_list.len() as i32 - 1 {
        let i:usize = job_index as usize;
        let mut valid: bool = true;
        // judge whether problem_id, language and times are corresponded
        let problem_id: Option<usize> = info.problem_id.clone();
        let language: Option<String> = info.language.clone();
        let from: Option<String> = info.from.clone();
        let to: Option<String> = info.to.clone();
        let state: Option<State> = info.state.clone();
        let result: Option<Result> = info.result.clone();
        if problem_id.is_some() == true {
            if job_list[i].submission.problem_id != problem_id.unwrap() {
                valid = false;
            }
        } // problem_id
        if language.is_some() == true && valid == true {
            if job_list[i].submission.language != language.unwrap() {
                valid = false;
            }
        } // language
        if from.is_some() == true && valid == true {
            let from_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &from.unwrap(), "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            let created_time: String = job_list[i].created_time.clone();
            let created_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &created_time, "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            // convert `String` to `NaiveDateTime` type for comparison
            if created_instant < from_instant {
                valid = false;
            }
        } // from
        if to.is_some() == true && valid == true {
            let to_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &to.unwrap(), "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            let created_time: String = job_list[i].created_time.clone();
            let created_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &created_time, "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            // convert `String` to `NaiveDateTime` type for comparison
            if created_instant > to_instant {
                valid = false;
            }
        } // to
        if state.is_some() == true && valid == true {
            if &state.unwrap() != &job_list[i].state {
                valid = false;
            }
        } // state
        if result.is_some() == true && valid == true {
            if &result.unwrap() != &job_list[i].result {
                valid = false;
            }
        } // result
        if valid == true {
            response.push(job_list[i].clone());
        }
        job_index += 1; // search the next job
    }

    drop(job_lock);
    update_json_file();

    HttpResponse::Ok().json(response)
}
