use std::mem::swap;

use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::global::{USER_LIST, GLOBAL_CONTEST_LIST, User};
use crate::config::Config;
use crate::response;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScoringRule {
    Latest,
    Highest,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TieBreaker {
    SubmissionTime,
    SubmissionCount,
    UserId,
}

#[derive(Serialize, Deserialize)]
struct RankListArg {
    scoring_rule: Option<ScoringRule>,
    tie_breaker: Option<TieBreaker>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct RankListResponse {
    user: User,
    rank: usize,
    scores: Vec<f64>,
}

#[get("/contests/{contest_id}/ranklist")]
pub async fn get_contests_ranklist(path: web::Path<usize>, 
    info: web::Query<RankListArg>,
    config: web::Data<Config>) -> impl Responder {

    let arguments = info.into_inner();
    let user_lock = USER_LIST.lock().unwrap();
    let user_num = (*user_lock).len(); // the number of users
    let problem_num = config.problems.len();
    // Note that the code of problems in not necessarily continuous

    let mut response: Vec<RankListResponse> = 
        vec![RankListResponse::default(); user_num];
    // println!("{}", user_num);

    let mut user_score: Vec<Vec<f64>> = Vec::new();
    let mut user_id: Vec<usize> = Vec::new();
    for i in 0..user_num {
        user_score.push(vec![0.0; problem_num + 1]);
        user_id.push(i);
    } // initialize the 2-dimension vector
    // user_score (i, j) means user_id = i, and the No.j problem
    // the (i, problem_num) position store the total scores

    let mut rule: ScoringRule = ScoringRule::Latest;
    if arguments.scoring_rule.is_some() == true {
        rule = arguments.scoring_rule.clone().unwrap();
    }

    let mut user_submit_order: Vec<i32> = vec![-1; user_num];
    let mut user_submit_count: Vec<i32> = vec![0; user_num];

    if rule == ScoringRule::Latest { // scoring_rule = latest

            let global_contest_list = 
                GLOBAL_CONTEST_LIST.lock().unwrap();
            let len = (*global_contest_list).len();
            for i in 0..len { // check all the submission records
                let user_id = (*global_contest_list)[i].user_id;
                let problem_id = (*global_contest_list)[i].problem_id;
                let mut problem_index: usize = 0; 
                // the problem index in user_score vector

                for j in 0..config.problems.len() {
                    if config.problems[j].id == problem_id as u64 {
                        problem_index = j;
                        break;
                    }
                } // get the index for problem in user_score vector

                user_score[user_id][problem_index] = 
                    (*global_contest_list)[i].score;
                user_submit_order[user_id] = i as i32; // as the time order
                // record the latest submit
                user_submit_count[user_id] += 1;
            }

            for i in 0..user_num {
                for j in 0..problem_num {
                    user_score[i][problem_num] += user_score[i][j];
                }
            } // calculate the total scores

            for i in 1 ..= user_num - 1 {
                let mut j = i - 1;
                loop { // compare j and j + 1 (j/j + 1 isn't the user id)
                    if user_score[j][problem_num] < 
                        user_score[j + 1][problem_num] {
                            let t = user_score[j].clone();
                            user_score[j] = user_score[j + 1].clone();
                            user_score[j + 1] = t.clone();
                            let u = user_id[j];
                            user_id[j] = user_id[j + 1];
                            user_id[j + 1] = u; // swap
                        }
                    if user_score[j][problem_num] == 
                        user_score[j + 1][problem_num] 
                        && arguments.tie_breaker.clone().is_some() == true {

                        let breaker = arguments.tie_breaker
                            .clone().unwrap();
                        if breaker == TieBreaker::SubmissionTime {
                            if user_submit_order[user_id[j]] >
                                    user_submit_order[user_id[j + 1]] {
                                let t = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap
                            }
                        }
                        if breaker == TieBreaker::SubmissionCount {
                            if user_submit_count[user_id[j]] >
                                    user_submit_count[user_id[j + 1]] {
                                let t = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap    
                            }
                        }
                    
                    } // use tie-breakers
                    
                    if j == 0 { break; }
                    j -= 1;
                }
            } // bubble sort

            // println!("user_num = {}", user_num);
            for i in 0..user_num {
                response[i].user.id = user_id[i];
                response[i].user.name = (*user_lock)[user_id[i]].name.clone();
                response[i].rank = i + 1;
                for j in 0..problem_num {
                    response[i].scores.push(user_score[i][j]);
                }
                if i > 0 { // from 1 to user_num - 1
                    if user_score[i][problem_num] == 
                        user_score[i - 1][problem_num] {
                            let mut b = true;
                            if arguments.tie_breaker.clone().is_some() == true {
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionCount {
                                        if user_submit_count[user_id[i - 1]] !=
                                        user_submit_count[user_id[i]] {
                                            b = false;
                                        }
                                    }
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::UserId {
                                    b = false;
                                }
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionTime {
                                    b = false;
                                }
                            }
                            if b == true {
                                response[i].rank = response[i - 1].rank;
                            }
                        }
                }
            }

        } // scoring_rule = latest
    else {

        let global_contest_list = 
                GLOBAL_CONTEST_LIST.lock().unwrap();
            let len = (*global_contest_list).len();
            let mut find_score_submit_order: Vec<Vec<i32>> = 
                vec![vec![-1; problem_num]; user_num];
            for i in 0..len {
                let user_id = (*global_contest_list)[i].user_id;
                let problem_id = (*global_contest_list)[i].problem_id;
                let mut problem_index: usize = 0; 
                // the problem index in user_score vector

                for j in 0..config.problems.len() {
                    if config.problems[j].id == problem_id as u64 {
                        problem_index = j;
                        break;
                    }
                } // get the index for problem in user_score vector

                if user_score[user_id][problem_index] < 
                    (*global_contest_list)[i].score {
                        user_score[user_id][problem_index] = 
                            (*global_contest_list)[i].score;
                        find_score_submit_order[user_id][problem_id] = i as i32;
                    } // find the highest score
                user_submit_count[user_id] += 1;
            }

            for i in 0..user_num {
                for j in 0..problem_num {
                    if find_score_submit_order[i][j] > user_submit_order[i] {
                        user_submit_order[i] = 
                            find_score_submit_order[i][j];
                    }
                }
            } // get the latest submit time for all users

            for i in 0..user_num {
                for j in 0..problem_num {
                    user_score[i][problem_num] += user_score[i][j];
                }
            } // calculate the total scores

            for i in 1 ..= user_num - 1 {
                let mut j = i - 1;
                loop { // compare j and j + 1
                    if user_score[j][problem_num] < 
                        user_score[j + 1][problem_num] {
                            let t = user_score[j].clone();
                            user_score[j] = user_score[j + 1].clone();
                            user_score[j + 1] = t.clone();
                            let u = user_id[j];
                            user_id[j] = user_id[j + 1];
                            user_id[j + 1] = u;
                        } 

                    if user_score[j][problem_num] == 
                        user_score[j + 1][problem_num] 
                        && arguments.tie_breaker.clone().is_some() == true {

                        let breaker = arguments.tie_breaker
                            .clone().unwrap();
                        if breaker == TieBreaker::SubmissionTime {
                            if user_submit_order[user_id[j]] >
                                user_submit_order[user_id[j + 1]] {
                                    let t = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap
                            }
                        }
                        if breaker == TieBreaker::SubmissionCount {
                            if user_submit_count[user_id[j]] >
                                    user_submit_count[user_id[j + 1]] {
                                let t = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap    
                            }
                        }
                    
                    } // use tie-breakers

                    if j == 0 { break; }
                    j -= 1;
                }
            } // bubble sort

            // println!("user_num = {}", user_num);
            for i in 0..user_num {
                response[i].user.id = user_id[i];
                response[i].user.name = (*user_lock)[user_id[i]].name.clone();
                response[i].rank = i + 1;
                for j in 0..problem_num {
                    response[i].scores.push(user_score[i][j]);
                }
                if i > 0 {
                    if user_score[i][problem_num] == 
                        user_score[i - 1][problem_num] {
                            let mut b = true;
                            if arguments.tie_breaker.clone().is_some() == true {
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionCount {
                                        if user_submit_count[user_id[i - 1]] !=
                                        user_submit_count[user_id[i]] {
                                            b = false;
                                        }
                                    }
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::UserId {
                                    b = false;
                                }
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionTime {
                                    b = false;
                                }
                            }
                            if b == true {
                                response[i].rank = response[i - 1].rank;
                            }
                        }
                }
            }
    }

    HttpResponse::Ok().json(response)
}