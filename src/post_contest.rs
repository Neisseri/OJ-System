use std::sync::MutexGuard;
use actix_web::{post, web, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::global::{CONTEST_INFO, Contest, USER_LIST, User};
use crate::error::Error;
use crate::config::Config;
use crate::persistent_storage::update_json_file;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PostContest {
    pub id: Option<usize>,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<usize>,
    pub user_ids: Vec<usize>,
    pub submission_limit: u64,
}

#[post("/contests")]
async fn post_contests(body: web::Json<PostContest>,
    config: web::Data<Config>) -> impl Responder {

    let mut contest_info: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let contest_num: usize = contest_info.len(); // id from 1 to contest_num
    let user_list: MutexGuard<Vec<User>> = USER_LIST.lock().unwrap();
    let user_num: usize = (*user_list).len();
    if body.id.clone().is_some() == true { 
        // find the contest and renew the information
        let contest_id = body.id.clone().unwrap();
        if contest_id > contest_num { // the id is invalid
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Contest 114514 not found.".to_string(),
            });
        } // the contest id is not existed, return 404 error
        // if the id is valid, find it and renew the information
        let mut user_and_problem_valid: bool = true;
        // whether the user_id and problem_id is valid
        // Note that the Id for problems is not necessarily continuous
        let user_ids: Vec<usize> = body.user_ids.clone();
        for i in 0..user_ids.len() {
            if user_ids[i] >= user_num {
                // the valid user_id is from 0 to user_num - 1
                user_and_problem_valid = false;
                break;
            }
        } // check the validity of user_id
        let problem_ids: Vec<usize> = body.problem_ids.clone();
        for i in 0..problem_ids.len() {
            let mut if_find_problem_id: bool = false;
            for j in 0..config.problems.len() {
                if problem_ids[i] == config.problems[j].id as usize {
                    if_find_problem_id = true;
                    break;
                }
            } // search the problem_id in the configuration
            if if_find_problem_id == false {
                user_and_problem_valid = false;
                break;
            }
        } // check the validity of problem_id
        if user_and_problem_valid == false {
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Contest 114514 not found.".to_string(),
            });
        } // if the problem or user is not existed, return 404 error
        // as the `id = 0` is particular, `id = i` means `contest_info[i - 1]` 
        // the fields that need to be renewed:
        // name, from, to, problem_ids, user_ids, submission_limit
        (*contest_info)[contest_id - 1].name = body.name.clone();
        (*contest_info)[contest_id - 1].from = body.from.clone();
        (*contest_info)[contest_id - 1].to = body.to.clone();
        (*contest_info)[contest_id - 1].problem_ids = body.problem_ids.clone();
        (*contest_info)[contest_id - 1].user_ids = body.user_ids.clone();
        (*contest_info)[contest_id - 1].submission_limit = body.submission_limit.clone();
        update_json_file();
        return HttpResponse::Ok().json((*contest_info)[contest_id - 1].clone());
    } // `Id` is provided
    else { // create a new contest
        // First, check whether user_ids and problem_ids are valid
        let mut user_and_problem_valid: bool = true;
        // whether the user_id and problem_id is valid
        // Note that the Id for problems is not necessarily continuous
        let user_ids: Vec<usize> = body.user_ids.clone();
        for i in 0..user_ids.len() {
            if user_ids[i] >= user_num {
                // the valid user_id is from 0 to user_num - 1
                user_and_problem_valid = false;
                break;
            }
        } // check the validity of user_id
        let problem_ids: Vec<usize> = body.problem_ids.clone();
        for i in 0..problem_ids.len() {
            let mut if_find_problem_id: bool = false;
            for j in 0..config.problems.len() {
                if problem_ids[i] == config.problems[j].id as usize {
                    if_find_problem_id = true;
                    break;
                }
            } // search the problem_id in the configuration
            if if_find_problem_id == false {
                user_and_problem_valid = false;
                break;
            }
        } // check the validity of problem_id
        if user_and_problem_valid == false {
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Contest 114514 not found.".to_string(),
            });
        } // if the problem or user is not existed, return 404 error
        // The existing contest-ids are from 1 to contest_num
        // push the value of `id = contest_num + 1`
        let new_contest: Contest = 
            Contest { 
                id: contest_num + 1, 
                name: body.name.clone(), 
                from: body.from.clone(), 
                to: body.to.clone(), 
                problem_ids: body.problem_ids.clone(), 
                user_ids: body.user_ids.clone(), 
                submission_limit: body.submission_limit.clone(), 
            };
        (*contest_info).push(new_contest.clone());

        drop(contest_info);
        drop(user_list);
        update_json_file();
        return HttpResponse::Ok().json(new_contest.clone());
    } // `Id` is not provided
}