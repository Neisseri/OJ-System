pub mod post_job_api {

    use actix_web::{post, web, Responder, HttpResponse};
    use serde::{Serialize, Deserialize};
    use crate::config::Config;
    use crate::error::Error;
    use crate::global;
    use std::fs;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use crate::response::{Response, CaseResult, Result, State};
    use std::time::{Duration, Instant};
    use chrono::{Utc, SecondsFormat};
    use crate::tool::vec_char_equal;
    use wait_timeout::ChildExt;

    #[derive(Clone, Deserialize, Serialize, Default, Debug)]
    pub struct PostJob {
        pub source_code: String,
        pub language: String,
        pub user_id: u64,
        pub contest_id: u64,
        pub problem_id: u64
    }

    #[post("/jobs")]
    async fn post_jobs(body: web::Json<PostJob>, 
        config: web::Data<Config>) -> impl Responder {

        let mut response = Response::new();
        // the json struct 'Response` type `response`
        response.created_time = Utc::now().
            to_rfc3339_opts(SecondsFormat::Millis, true);
        // println!("{}", &response.created_time);
        
        let language = &body.language;
        let langs = &config.languages;
        let mut file_name: String = String::new();
        let mut valid_language: bool = false;
        let mut command: String = String::new();
        let mut argumemts: Vec<String> = Vec::new();
        for i in 0..langs.len() {
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
        let mut pro_index: usize = 0;
        let pro_id = &body.problem_id;
        let problems = &config.problems;
        for i in 0..problems.len() {
            if pro_id == &problems[i].id {
                valid_problem_id = true;
                pro_index = i;
                break;
            }
        } // check the problem id
        let user_list = global::USER_LIST.lock().unwrap();
        if valid_language == false || valid_problem_id == false 
            || body.user_id > (*user_list).len() as u64 - 1 { // check the user id
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "HTTP 404 Not Found".to_string()
            });
        } // return the Error response

        let mut lock = global::JOB_NUM.lock().unwrap();
        *lock += 1;
        let job_num = *lock - 1; // get the global variable
        // the serial number for judge jobs, i.e. the judge_job Id
        response.id = job_num;

        let s = format!("./target/tmp/TMPDIR_{}", job_num).clone();
        // `s` is the path of the temporary directory
        // println!("PATH is {}", &s);
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
        // println!("{}", &out_file_path);
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

            let begin_instant = Instant::now();
            let in_file = fs::File::open(&cases[i].input_file).unwrap();
            let out_file = fs::File::create(&out_file_path).unwrap();
            /*let mut status = Command::new(&exe_path)
                    .stdin(Stdio::from(in_file))
                    .stdout(Stdio::from(out_file))
                    .stderr(Stdio::null())
                    .status();*/

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
            if &pro_info.r#type == "standard" {
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
                // println!("{}", &output);
                // println!("{}", &answer);
                cmp_result = vec_char_equal(&out, &ans);
            } else if &pro_info.r#type == "strict" {
                if answer == output {
                    cmp_result = true;
                } else {
                    cmp_result = false;
                }
            } // get the result of the comparison between output and answer

            if cmp_result == true {
                total_score += cases[i].score;
                response.cases.push(CaseResult { 
                    id: (i + 1) as u64, 
                    result: Result::Accepted, 
                    time: run_time as u64, 
                    memory: 0, 
                    info: "".to_string() 
                });
            } else {
                response.cases.push(CaseResult { 
                    id: (i + 1) as u64, 
                    result: Result::WrongAnswer, 
                    time: run_time as u64, 
                    memory: 0, 
                    info: "".to_string() 
                });
            }
        } // traverse the cases data

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

        let response_body = 
            serde_json::to_string_pretty(&response.clone())
            .unwrap();
        // change the struct to json format String

        // println!("{response_body}");

        let mut lock = global::JOB_LIST.lock().unwrap();
        (*lock).push(response.clone());

        let mut global_contest_list = global::GLOBAL_CONTEST_LIST.lock().unwrap();
        (*global_contest_list).push(global::Submit {
            user_id: body.user_id as usize,
            problem_id: body.problem_id as usize,
            score: total_score,
            submit_time: response.updated_time.clone(),
        });

        HttpResponse::Ok().body(response_body)
    }
}

