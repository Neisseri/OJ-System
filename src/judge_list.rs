use actix_web::{get, web, Responder, HttpResponse};
use crate::response::{State, Result, Response};
use serde::{Serialize, Deserialize};
use crate::global::JOB_LIST;
use chrono::NaiveDateTime;
use crate::persistent_storage::update_json_file;

#[derive(Serialize, Deserialize)]
struct ListFilter {
    user_id: Option<u64>,
    user_name: Option<String>,
    contest_id: Option<u64>,
    problem_id: Option<u64>,
    language: Option<String>,
    from: Option<String>,
    to: Option<String>,
    state: Option<State>,
    result: Option<Result>,
}

#[get("/jobs")]
async fn get_jobs(info: web::Query<ListFilter>) -> impl Responder {

    let lock = JOB_LIST.lock().unwrap();
    let get_jobs_json = (*lock).clone();
    let mut filtered_list: Vec<Response> = Vec::new();
    // obey the descending order for times
    let mut index: i32 = 0;
    while index <= get_jobs_json.len() as i32 - 1 {
        let i = index as usize;
        let mut valid: bool = true;
        let problem_id = info.problem_id.clone();
        let language = info.language.clone();
        let from = info.from.clone();
        let to = info.to.clone();
        let state = info.state.clone();
        let result = info.result.clone();
        if problem_id.is_some() == true {
            if get_jobs_json[i].submission.problem_id != problem_id.unwrap() {
                valid = false;
            }
        } // problem_id
        if language.is_some() == true && valid == true {
            if get_jobs_json[i].submission.language != language.unwrap() {
                valid = false;
            }
        } // language
        if from.is_some() == true && valid == true {
            let from_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &from.unwrap(), "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            let created_time: String = get_jobs_json[i].created_time.clone();
            let created_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &created_time, "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            if created_instant < from_instant {
                valid = false;
            }
        } // from
        if to.is_some() == true && valid == true {
            let to_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &to.unwrap(), "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            let created_time: String = get_jobs_json[i].created_time.clone();
            let created_instant: NaiveDateTime = NaiveDateTime::parse_from_str(
                &created_time, "%Y-%m-%dT%H:%M:%S%.3fZ"
            ).unwrap();
            if created_instant > to_instant {
                valid = false;
            }
        } // to
        if state.is_some() == true && valid == true {
            if &state.unwrap() != &get_jobs_json[i].state {
                valid = false;
            }
        } // state
        if result.is_some() == true && valid == true {
            if &result.unwrap() != &get_jobs_json[i].result {
                valid = false;
            }
        } // result
        if valid == true {
            filtered_list.push(get_jobs_json[i].clone());
        }
        index += 1;
    }
    let response_body = 
            serde_json::to_string_pretty(&filtered_list.clone())
            .unwrap();
    // println!("{response_body}");

    drop(lock);
    update_json_file();

    HttpResponse::Ok().body(response_body)
}
