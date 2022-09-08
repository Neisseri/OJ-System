use actix_web::{post, web, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::config::{Config, Language, Problem};
use crate::error::Error;
use crate::global::{User, USER_LIST, JOB_NUM, 
        JOB_LIST, GLOBAL_CONTEST_LIST, Submit, CONTEST_INFO,
        Contest};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::MutexGuard;
use crate::response::{Response, CaseResult, Result, State};
use std::time::{Duration, Instant};
use chrono::{Utc, SecondsFormat};
use crate::tool::vec_char_equal;
use wait_timeout::ChildExt;
use crate::persistent_storage::update_json_file;

#[derive(Clone, Deserialize, Serialize, Default, Debug)]
pub struct PostJob {
    pub source_code: String,
    pub language: String,
    pub user_id: usize,
    pub contest_id: usize,
    pub problem_id: usize,
}

#[post("/jobs")]
async fn post_jobs(body: web::Json<PostJob>, 
    config: web::Data<Config>) -> impl Responder {

    let tmp_dir_path: &str = "./target/tmp";
    fs::remove_dir_all(tmp_dir_path).unwrap();
    fs::create_dir(tmp_dir_path).unwrap();
    // remove all the files in tmp but not the directory

    let mut response: Response = Response::new();
    // the json struct 'Response` type `response`
    response.created_time = Utc::now().
        to_rfc3339_opts(SecondsFormat::Millis, true);
        
    let language: &String = &body.language;
    let langs: &Vec<Language> = &config.languages; // the language list
    let mut file_name: String = String::new();
    let mut valid_language: bool = false; // if the language is valid?
    let mut command: String = String::new();
    let mut argumemts: Vec<String> = Vec::new();
    for i in 0..langs.len() { // serach the language list
        if language == &langs[i].name {
            valid_language = true;
            file_name = langs[i].file_name.clone();
            // the name of source code file, such as `main.rs`
            let l = langs[i].command.len();
            for j in 0..l {
                if j == 0 {
                    command = langs[i].command[j].clone();
                } else {
                    argumemts.push(langs[i].command[j].clone());
                }
            }
            break;
        }
    } // check the language

    let mut valid_problem_id: bool = false;
    let mut pro_index: usize = 0; // the index in the problem list
    let pro_id: &usize = &body.problem_id; // submit problem id
    let problems: &Vec<Problem> = &config.problems; // problem list
    for i in 0..problems.len() { // search the problem list
        if pro_id == &problems[i].id { // find the problem
            valid_problem_id = true;
            pro_index = i; // get the index for the problem
            break;
        }
    } // check the problem id
    let user_list: MutexGuard<Vec<User>> = USER_LIST.lock().unwrap();
    // stores the information of all the users
    if valid_language == false || valid_problem_id == false 
        || body.user_id > (*user_list).len() - 1 { 
        // language or problem_id or user_id is invalid
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "HTTP 404 Not Found".to_string()
        });
    } // return the Error response

    // Since the problem_id and user_id are valid,
    // next we should check whether the contest_id is valid
    // and whether the problem and user are in the contest

    let mut if_wrong: Vec<bool> = Vec::new(); // for the packing judge
    // if the group have appeared wrong cases
    // the advanced requirements of packing judge
    let mut in_which_group: usize = 0;
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut if_packing: bool = false;
    if problems[pro_index].misc.is_some() == true {
        let misc = problems[pro_index].misc.clone().unwrap();
        if misc.packing.is_some() == true {
            if_packing = true;
            groups = misc.packing.clone().unwrap();
            for _ in 0..groups.len() {
                if_wrong.push(false);
            }
        }
    } // Packing Judge

    let contest_id: usize = body.contest_id;
    if contest_id != 0 {
        let contest_lock: MutexGuard<Vec<crate::global::Contest>> = 
            CONTEST_INFO.lock().unwrap();
        let contest_info: Vec<Contest> = (*contest_lock).clone();
        let contest_num: usize = contest_info.len();
        if contest_id > contest_num {
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "HTTP 404 Not Found".to_string(),
            });
        } // the contest_id is invalid, return 404 Not Found Error

        let mut find_user: bool = false;
        let mut find_problem: bool = false;
        let target_contest: Contest = 
            contest_info[contest_id as usize - 1].clone();
        for i in 0..target_contest.user_ids.len() {
            if body.user_id as usize == target_contest.user_ids[i] {
                find_user = true;
                break;
            }
        } // search the user_id list of this contest
        for i in 0..target_contest.problem_ids.len() {
            if body.problem_id as usize == target_contest.problem_ids[i] {
                find_problem = true;
                break;
            }
        } // search the problem_id list of this contest
        if find_user == false || find_problem == false {
            return HttpResponse::BadRequest().json(Error {
                code: 1,
                reason: "ERR_INVALID_ARGUMENT".to_string(),
                message: "HTTP 400 Bad Request".to_string(),
            });
        }
            
        let contest_list_lock = GLOBAL_CONTEST_LIST.lock().unwrap();
        let global_contest_list = (*contest_list_lock).clone();
        let mut have_submit_time: u64 = 0;
        for i in 0..global_contest_list.len() {
            if body.user_id as usize == global_contest_list[i].user_id
                && body.problem_id as usize == global_contest_list[i].problem_id {
                    
                have_submit_time += 1;
            }
        } // the times have submitted for this problem
        if have_submit_time >= target_contest.submission_limit {
            return HttpResponse::BadRequest().json(Error {
                code: 4,
                reason: "ERR_RATE_LIMIT".to_string(),
                message: "HTTP 400 Bad Request".to_string(),
            });
        } // submit limites invalid
    }
    // advanced requirements: contest support

    let mut lock = JOB_NUM.lock().unwrap();
    *lock += 1;
    let job_num = *lock - 1; // get the global variable
    // the serial number for judge jobs, i.e. the judge_job Id
    response.id = job_num;

    let s = format!("./target/tmp/TMPDIR_{}", job_num).clone();
    // `s` is the path of the temporary directory

    let result = fs::create_dir(s.clone());
    if result.is_err() == true {
        return HttpResponse::BadRequest().json(Error {
            code: 5,
            reason: "ERR_EXTERNAL".to_string(),
            message: "HTTP 500 Internal Server Error".to_string()
        });
    } // create a new temporary directory

    let file_path = format!("{}/{}", s.clone(), file_name.clone());
    // the path of source code file
    let mut file = fs::File::create(file_path.clone()).unwrap();
    let source_code = body.source_code.clone();
    file.write_all(source_code.as_bytes()).unwrap();
    // write the source code to the file

    let mut exe_file_name: String = "test".to_string();
    if cfg!(target_os = "Windows") { exe_file_name = "test.exe".to_string(); }
    // get the name for the execute file

    let exe_path = format!("{}/{}", s.clone(), exe_file_name.clone()).clone();
    // get the path for execute file
    for i in 0..argumemts.len() {
        if argumemts[i] == "%INPUT%".to_string() {
            argumemts[i] = file_path.clone();
        } else if argumemts[i] == "%OUTPUT%" {
            argumemts[i] = exe_path.clone();
        } 
    } // replace the arguments with source-code path and exe path

    let status = Command::new(command.clone())
        .args(argumemts)
        .status();
    // compile the source code and create execute file
    if status.unwrap().success() == true {
        response.cases.push(CaseResult {
            id: 0,
            result: Result::CompilationSuccess,
            time: 0,
            memory: 0,
            info: "".to_string()
        });
    } else {
        response.cases.push(CaseResult { 
            id: 0, 
            result: Result::CompilationError, 
            time: 0, 
            memory: 0, 
            info: "".to_string()
        });
        response.result = Result::CompilationError;
    } // push the result of compilation
        
    // The Problem index in problems vector: pro_index
    let pro_info = config.problems[pro_index].clone();
    let cases = pro_info.cases.clone();
    let out_file_path = format!("{}/test.out", s.clone()).clone();

    let mut total_score: f64 = 0.0;
    for i in 0..cases.len() {

        if response.result == Result::CompilationError {
            response.cases.push(CaseResult { 
                id: (i + 1) as u64, 
                result: Result::Waiting, 
                time: 0, 
                memory: 0, 
                info: "".to_string() 
            });
            continue;
        } // if compile error, the result is `waiting`

        let in_file = fs::File::open(&cases[i].input_file).unwrap();
        let out_file = fs::File::create(&out_file_path).unwrap();
            
        let begin_instant = Instant::now();
        let mut child = Command::new(&exe_path)
            .stdin(Stdio::from(in_file))
            .stdout(Stdio::from(out_file))
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let wait_time = Duration::from_micros(500000 + cases[i].time_limit);
        let status = 
            match child.wait_timeout(wait_time).unwrap() {
                Some(status) => status,
                 None => {
                    // child hasn't exited yet
                    child.kill().unwrap();
                    response.cases.push(CaseResult {
                        id: (i + 1) as u64,
                        result: Result::TimeLimitExceeded,
                        time: 500 + cases[i].time_limit,
                        memory: 0,
                        info: "".to_string()
                    });
                    response.result = Result::TimeLimitExceeded;
                    continue;
                }
            }; 

        // generate the output file
        let end_instant = Instant::now();
        let run_time = end_instant.
            duration_since(begin_instant)
            .as_micros();
        // get the run time

        if status.success() == false {
            response.cases.push(CaseResult { 
                id: (i + 1) as u64, 
                result: Result::RuntimeError, 
                time: run_time as u64, 
                memory: 0, 
                info: "".to_string() 
            });
            response.result = Result::RuntimeError;
            continue;
        } // Runtime Error: such as the program panic
 
        let answer = std::fs::read_to_string(&cases[i].answer_file).unwrap();
        let output = std::fs::read_to_string(&out_file_path).unwrap();
        let mut cmp_result: bool = true;

        let mut info_s: String = String::new();

        if &pro_info.r#type == "standard" { // Standard Judge
            let mut ans: Vec<char> = Vec::new();
            for c in answer.clone().chars() {
                if c == '\n' {
                    while ans.last().unwrap() == &' ' {
                        ans.pop(); // remove the blanks in line end
                    }
                }
                ans.push(c);
            }
            while ans.last().unwrap() == &'\n' {
                ans.pop();
            } // remove the empty lines in file end
            let mut out: Vec<char> = Vec::new();
            for c in output.clone().chars() {
                if c == '\n' {
                    while out.last().unwrap() == &' ' {
                        out.pop(); // remove the blanks in line end
                    }
                }
                out.push(c);
            }
            while out.last().unwrap() == &'\n' {
                out.pop();
            } // remove the empty lines in file end

            cmp_result = vec_char_equal(&out, &ans);
        } else if &pro_info.r#type == "strict" { // Strict 
            if answer == output {
                cmp_result = true;
            } else {
                cmp_result = false;
            }
        } // get the result of the comparison between output and answer
        else if &pro_info.r#type == "spj" { // Special Judge
            let spj: Vec<String> = 
                pro_info.clone().misc.unwrap().special_judge.unwrap();
            let mut cmd: String = String::new();
            let mut args: Vec<String> = Vec::new();
            let mut b: bool = false;
            for ss in spj.into_iter() {
                if b == false {
                    cmd = ss.clone();
                    b = true;
                } else {
                    if &ss == "%OUTPUT%" {
                        args.push(out_file_path.clone());
                    } else if &ss == "%ANSWER%" {
                        args.push(cases[i].answer_file.clone());
                    } else {
                        args.push(ss.clone());
                    }
                }
            } // get the arguments for Run Command
            let output = Command::new(cmd.clone())
                .args(args.clone())
                .output().unwrap().stdout;
            let result = String::from_utf8(output).unwrap();
            let mut ch_res: Vec<char> = Vec::new();                           
            for ch in result.clone().chars() {
                ch_res.push(ch);
            }
            if ch_res[0] == 'A' {
                cmp_result = true;
            } else {
                cmp_result = false;
            }
            let mut start_read = false;
            for ch in result.clone().chars() {
                if ch == '\n' {
                    start_read = true;
                }
                if start_read == true && ch != '\n' {
                    info_s.push(ch.clone());
                }
            }
        }

        if if_packing == true { // the Packing Judge
            if if_wrong[in_which_group] == true {
                response.cases.push(CaseResult { 
                    id: (i + 1) as u64, 
                    result: Result::Skipped, 
                    time: run_time as u64, 
                    memory: 0, 
                    info: info_s.clone(),
                });
            } // skip the group
            else {
                if cmp_result == true {
                    response.cases.push(CaseResult { 
                        id: (i + 1) as u64, 
                        result: Result::Accepted, 
                        time: run_time as u64, 
                        memory: 0, 
                        info: info_s.clone(),
                    });
                } else {
                    response.cases.push(CaseResult { 
                        id: (i + 1) as u64, 
                        result: Result::WrongAnswer, 
                        time: run_time as u64, 
                        memory: 0, 
                        info: info_s.clone(),
                    });
                    if_wrong[in_which_group] = true;
                }
            }
            if groups[in_which_group].last().unwrap() == &((i + 1) as usize) {
                in_which_group += 1;
            }
        } else { // the Normal Judge

            if cmp_result == true {
                total_score += cases[i].score;
                response.cases.push(CaseResult { 
                    id: (i + 1) as u64, 
                    result: Result::Accepted, 
                    time: run_time as u64, 
                    memory: 0, 
                    info: info_s.clone(),
                });
            } else {
                response.cases.push(CaseResult { 
                    id: (i + 1) as u64, 
                    result: Result::WrongAnswer, 
                    time: run_time as u64, 
                    memory: 0, 
                    info: info_s.clone(),
                });
            }
        }

    } // traverse the cases data

    if if_packing == true { // Packing Judge
        total_score = 0.0;
        for i in 0..groups.len() {
            if if_wrong[i] == false { // the group is Correct
                for j in 0..groups[i].len() {
                    let case_index = groups[i][j] - 1;
                    total_score += problems[pro_index].cases[case_index].score;
                }
            }
        }
    }

    response.score = total_score;
    if total_score == 100.0 {
        response.result = Result::Accepted;
    } else {
        if response.result == Result::Default {
            response.result = Result::WrongAnswer;
        }   
    }
    response.state = State::Finished;
    response.submission = body.clone();
    // assign the value of score, result, state, submission 

    let result = fs::remove_dir_all(s.clone());
    if result.is_err() == true {
        return HttpResponse::BadRequest().json(Error {
            code: 5,
            reason: "ERR_EXTERNAL".to_string(),
            message: "HTTP 500 Internal Server Error".to_string()
        });
    } // remove the temporary directory

    response.updated_time = Utc::now().
        to_rfc3339_opts(SecondsFormat::Millis, true);
    // generate the updated time

    let mut lock = JOB_LIST.lock().unwrap();
    (*lock).push(response.clone());

    let mut run_time_vec: Vec<u64> = Vec::new();
    for i in 1..response.cases.len() {
        run_time_vec.push(response.cases[i].time.clone());
    } // get the RunTime list

    let mut global_contest_list = GLOBAL_CONTEST_LIST.lock().unwrap();
    (*global_contest_list).push(Submit {
        user_id: body.user_id as usize,
        problem_id: body.problem_id as usize,
        score: total_score,
        submit_time: response.updated_time.clone(),
        run_time: run_time_vec,
    });

    drop(global_contest_list);
    drop(lock);
    drop(user_list);

    update_json_file();

    HttpResponse::Ok().json(response)
}