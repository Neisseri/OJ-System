use std::sync::MutexGuard;

use actix_web::{get, web, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::error::Error;
use crate::global::{USER_LIST, GLOBAL_CONTEST_LIST, 
    User, CONTEST_INFO, Contest, Submit};
use crate::config::Config;
use crate::persistent_storage::update_json_file;

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
pub struct RankListArg {
    pub scoring_rule: Option<ScoringRule>,
    pub tie_breaker: Option<TieBreaker>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct RankListResponse {
    pub user: User,
    pub rank: usize,
    pub scores: Vec<f64>,
}

#[get("/contests/{contest_id}/ranklist")]
pub async fn get_contests_ranklist(path: web::Path<usize>, 
    info: web::Query<RankListArg>,
    config: web::Data<Config>) -> impl Responder {

    let arguments: RankListArg = info.into_inner(); // ranking rules
    let contest_id: usize = path.into_inner(); // the purpose contest
    let contest_info_lock: MutexGuard<Vec<Contest>> = CONTEST_INFO.lock().unwrap();
    let contest_info: Vec<Contest> = (*contest_info_lock).clone();
    // get the information for all the contests
    let contest_num: usize = contest_info.len();
    // the number of all the contests
    if contest_id > contest_num && contest_id != 0 {
        // the valid contest_id is from 1 to contest_num
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: format!("Contest {} not found.", contest_id).clone(),
        }); // return the 404 Not Found Error
    } // The contest id in INVALID

    let mut target_contest: Contest = Contest::default();
    if contest_id != 0 {
        target_contest = contest_info[contest_id - 1].clone();
    }
    // get the target contest's information
    // only use this variable in advanced requirements

    let user_lock: MutexGuard<Vec<User>> = USER_LIST.lock().unwrap(); // user_list
    let mut user_num: usize = (*user_lock).len(); // the number of users
    // the user_list stores the information of all the users
    let mut problem_num: usize = config.problems.len();
    // Note that the code of problems in not necessarily continuous
    // It stores all the problems in all the contests
    // We can't find which contest the problem is in from `config`

    if contest_id != 0 {
        user_num = target_contest.user_ids.len();
        problem_num = target_contest.problem_ids.len();
    } // get the user_num and problem_num in the appointed contest
    let mut response: Vec<RankListResponse> = 
        vec![RankListResponse::default(); user_num];
    // the user_num should be the users in the contest
    // remember to use the index as `contest_id - 1`
    // println!("{}", user_num);

    let mut user_score: Vec<Vec<f64>> = Vec::new();
    let mut user_use_time: Vec<Vec<Vec<u64>>> = Vec::new(); // for dynamic judging
    // user_score[i][j], user_num * problem_num, record user's scores
    let mut user_id: Vec<usize> = Vec::new();
    for i in 0..user_num {
        user_score.push(vec![0.0; problem_num + 1]);
        user_use_time.push(vec![vec![0]; problem_num]);
        if contest_id == 0 { user_id.push(i); }
        else {
            user_id.push(target_contest.user_ids[i]);
        } // push in the user_ids in the target contest
    } // initialize the 2-dimension vector
    // user_score (i, j) means user_id = i, and the No.j problem
    // the (i, problem_num) position store the total scores

    let mut rule: ScoringRule = ScoringRule::Latest;
    if arguments.scoring_rule.is_some() == true {
        rule = arguments.scoring_rule.clone().unwrap();
    } // the rule for calculating the scores

    let mut user_submit_order: Vec<i32> = vec![-1; user_num];
    // the latest submission's order for each user
    let mut user_submit_count: Vec<i32> = vec![0; user_num];
    // the total submission counting for each user

    if rule == ScoringRule::Latest { // scoring_rule = latest

            let global_contest_list: MutexGuard<Vec<Submit>> = 
                GLOBAL_CONTEST_LIST.lock().unwrap();
            // global_contest_list stores all the submit records
            let len: usize = (*global_contest_list).len();
            for i in 0..len { // check all the submission records
                let user_id: usize = (*global_contest_list)[i].user_id;
                // the user for this submission
                let problem_id: usize = (*global_contest_list)[i].problem_id;
                // the problem for this submission
                let mut problem_index: usize = 0; 
                // the problem index in user_score vector

                let mut find_user: i32 = -1;
                let mut find_problem: i32 = -1;
                if contest_id != 0 {
                    for j in 0..target_contest.user_ids.len() {
                        if target_contest.user_ids[j] == user_id {
                            find_user = j as i32;
                            break;
                        }
                    } // find the user's index in the contest's list
                    for j in 0..target_contest.problem_ids.len() {
                        if target_contest.problem_ids[j] == problem_id {
                            find_problem = j as i32;
                            break;
                        }
                    } // find the problem's index in the contest's list
                    if find_user == -1 || find_problem == -1 {
                        continue;
                    } // the user or the problem isn't in the target contest
                    // skip this submission
                }

                for j in 0..config.problems.len() {
                    if config.problems[j].id == problem_id as u64 {
                        problem_index = j;
                        break;
                    }
                } // get the index for problem in user_score vector
                // use `problem_index` if contest_id = 0, 
                // i.e. in basic requirements

                if contest_id == 0 {
                    user_score[user_id][problem_index] = 
                        (*global_contest_list)[i].score;
                    // give it the latest score
                    user_submit_order[user_id] = i as i32; 
                    // record the latest submit as the time order
                    user_submit_count[user_id] += 1;
                    user_use_time[user_id][problem_index] = 
                        (*global_contest_list)[i].run_time.clone();
                    // println!("$$${:#?}", (*global_contest_list)[i].run_time.clone());
                } else {
                    user_score[find_user as usize][find_problem as usize] = 
                        (*global_contest_list)[i].score;
                    user_use_time[find_user as usize][find_problem as usize] = 
                        (*global_contest_list)[i].run_time.clone();
                    // println!("$$${:#?}", (*global_contest_list)[i].run_time.clone());
                    user_submit_order[find_user as usize] = i as i32;
                    user_submit_count[find_user as usize] += 1;
                } // when the contest_id != 0

            } // traverse all the submissions

            // The Dynamic Ranking part
            // println!("{:#?}", user_use_time);
            for i in 0..problem_num {
                let pro_id = config.problems[i].id;
                let mut problem_index: usize = 0;
                for j in 0..config.problems.len() {
                    if config.problems[j].id == pro_id as u64 {
                        problem_index = j;
                        break;
                    }
                } // find the problem's index using its Id
                let case_num = config.problems[problem_index].cases.len();
                if &config.problems[problem_index].r#type == "dynamic_ranking" {
                    let radio = config.problems[problem_index].misc.clone()
                        .unwrap().dynamic_ranking_ratio.unwrap();
                    let mut min_time: Vec<u64> = Vec::new();
                    for j in 1..case_num + 1 {
                        let mut minn: u64 = 0;
                        for k in 0..user_num {
                            if minn == 0 || minn > user_use_time[k][i][j - 1] {
                                minn = user_use_time[k][i][j - 1];
                            }
                        } // search the min run time
                        min_time.push(minn);
                    }
                    for j in 0..user_num {
                        if user_score[j][i] < 100.0 {
                            user_score[j][i] = user_score[j][i] * (1.0 - radio);
                        } else {
                            // user: j, problem: problem_index (i), case k 
                            user_score[j][i] = user_score[j][i] * (1.0 - radio);
                            //println!("first{}", user_score[j][i]);
                            for k in 0..case_num {
                                let mut dyna_score = config.problems[problem_index]
                                    .cases[k].score * (1.0 - radio);
                                //println!("raw{}", dyna_score);
                                //println!("min = {}", min_time[k]);
                                dyna_score = dyna_score * (
                                    min_time[k] as f64 / 
                                        user_use_time[j][i][k] as f64);
                                user_score[j][i] += dyna_score;
                            }
                            //println!("last{}", user_score[j][i]);
                        }
                    }
                }
            }
            // The Dynamic Ranking part

            for i in 0..user_num {
                for j in 0..problem_num {
                    user_score[i][problem_num] += user_score[i][j];
                }
            } // calculate the total scores

            for i in 1 ..= user_num - 1 {
                let mut j: usize = i - 1;
                loop { // compare j and j + 1 (j/j + 1 isn't the user id)
                    if user_score[j][problem_num] < 
                        user_score[j + 1][problem_num] {
                            let t: Vec<f64> = user_score[j].clone();
                            user_score[j] = user_score[j + 1].clone();
                            user_score[j + 1] = t.clone();
                            let u: usize = user_id[j];
                            user_id[j] = user_id[j + 1];
                            user_id[j + 1] = u; // swap
                        }
                    if user_score[j][problem_num] == 
                        user_score[j + 1][problem_num] 
                        && arguments.tie_breaker.clone().is_some() == true {
                        // tie-breaking

                        let breaker: TieBreaker = arguments.tie_breaker
                            .clone().unwrap();
                        if breaker == TieBreaker::SubmissionTime {
                            if user_submit_order[user_id[j]] >
                                    user_submit_order[user_id[j + 1]] {
                                let t: Vec<f64> = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u: usize = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap
                            }
                        } // compare the submission time
                        if breaker == TieBreaker::SubmissionCount {
                            if user_submit_count[user_id[j]] >
                                    user_submit_count[user_id[j + 1]] {
                                let t: Vec<f64> = user_score[j].clone();
                                user_score[j] = user_score[j + 1].clone();
                                user_score[j + 1] = t.clone();
                                let u: usize = user_id[j];
                                user_id[j] = user_id[j + 1];
                                user_id[j + 1] = u; // swap    
                            }
                        } // compare the total submit times
                    
                    } // use tie-breakers
                    
                    if j == 0 { break; }
                    // We can't let j: usize == -1 by using `j -= 1`
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
                        user_score[i - 1][problem_num] { // There may be a draw
                            let mut b: bool = true;
                            if arguments.tie_breaker.clone().is_some() == true {
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionCount {
                                        if user_submit_count[user_id[i - 1]] !=
                                        user_submit_count[user_id[i]] {
                                            b = false;
                                        }
                                    } // compare submit times
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::UserId {
                                    b = false;
                                } // User Ids can't be the same
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionTime {
                                    b = false;
                                } // They can't submit at the same time
                            }
                            if b == true {
                                response[i].rank = response[i - 1].rank;
                            } // They are really in a draw
                        }
                }
            }

        } // scoring_rule = latest
        // remember to copy this to another file to modify the following codes
// -------------------------------------------------------------------
    else {

        let global_contest_list: MutexGuard<Vec<Submit>> = 
                GLOBAL_CONTEST_LIST.lock().unwrap();
            // global_contest_list stores all the submit records
            let len: usize = (*global_contest_list).len();
            let mut find_score_submit_order: Vec<Vec<i32>> = 
                vec![vec![-1; problem_num]; user_num];
            for i in 0..len { // check all the submission records
                let user_id = (*global_contest_list)[i].user_id;
                // the user for this submission
                let problem_id = (*global_contest_list)[i].problem_id;
                // the problem for this submission
                let mut problem_index: usize = 0; 
                // the problem index in user_score vector

                let mut find_user: i32 = -1;
                let mut find_problem: i32 = -1;
                if contest_id != 0 {
                    for j in 0..target_contest.user_ids.len() {
                        if target_contest.user_ids[j] == user_id {
                            find_user = j as i32;
                            break;
                        }
                    } // find the user's index in the contest's list
                    for j in 0..target_contest.problem_ids.len() {
                        if target_contest.problem_ids[j] == problem_id {
                            find_problem = j as i32;
                            break;
                        }
                    } // find the problem's index in the contest's list
                    if find_user == -1 || find_problem == -1 {
                        continue;
                    } // the user or the problem isn't in the target contest
                    // skip this submission
                }

                for j in 0..config.problems.len() {
                    if config.problems[j].id == problem_id as u64 {
                        problem_index = j;
                        break;
                    }
                } // get the index for problem in user_score vector
                // use `problem_index` if contest_id = 0, 
                // i.e. in basic requirements

                if contest_id == 0 {
                    if user_score[user_id][problem_index] < 
                    (*global_contest_list)[i].score {
                        user_score[user_id][problem_index] = 
                            (*global_contest_list)[i].score;
                        find_score_submit_order[user_id][problem_id] = i as i32;
                    } // find the highest score
                    user_submit_count[user_id] += 1;
                } else { // contest_id != 0
                    if user_score[find_user as usize][find_problem as usize] <
                        (*global_contest_list)[i].score {
                        user_score[find_user as usize][find_problem as usize] = 
                            (*global_contest_list)[i].score;
                        find_score_submit_order[find_user as usize][find_problem as usize] 
                            = i as i32;        
                    }
                }
                
            } // traverse all the submissions

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
                loop { // compare j and j + 1 (j/j + 1 isn't the user id)
                    if user_score[j][problem_num] < 
                        user_score[j + 1][problem_num] {
                            let t: Vec<f64> = user_score[j].clone();
                            user_score[j] = user_score[j + 1].clone();
                            user_score[j + 1] = t.clone();
                            let u = user_id[j];
                            user_id[j] = user_id[j + 1];
                            user_id[j + 1] = u; // swap
                        } 

                    if user_score[j][problem_num] == 
                        user_score[j + 1][problem_num] 
                        && arguments.tie_breaker.clone().is_some() == true {
                        // tie-breaking

                        let breaker: TieBreaker = arguments.tie_breaker
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
                        } // compare the submission time
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
                        } // compare the total submit times
                    
                    } // use tie-breakers

                    if j == 0 { break; }
                    // We can't let j: usize == -1 by using `j -= 1`
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
                        user_score[i - 1][problem_num] { // There may be a draw
                            let mut b: bool = true;
                            if arguments.tie_breaker.clone().is_some() == true {
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionCount {
                                        if user_submit_count[user_id[i - 1]] !=
                                        user_submit_count[user_id[i]] {
                                            b = false;
                                        }
                                    } // compare submit times
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::UserId {
                                    b = false; 
                                } // User Ids can't be the same
                                if arguments.tie_breaker.clone().unwrap()
                                    == TieBreaker::SubmissionTime {
                                    b = false;
                                } // They can't submit at the same time
                            }
                            if b == true {
                                response[i].rank = response[i - 1].rank;
                            } // They are really in a draw
                        }
                }
            }
    }

    drop(contest_info_lock);
    drop(user_lock);
    update_json_file();

    HttpResponse::Ok().json(response)
}