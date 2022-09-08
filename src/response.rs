use serde::{Serialize, Deserialize};
use crate::judge_task::PostJob;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: u64,
    pub created_time: String,
    pub updated_time: String,
    pub submission: PostJob,
    pub state: State,
    pub result: Result,
    pub score: f64,
    pub cases: Vec<CaseResult>,
}

impl Response {
    pub fn new() -> Response {
        Response { 
            id: 0, 
            created_time: String::new(),
            updated_time: String::new(), 
            submission: PostJob::default(),
            state: State::Default, 
            result: Result::Default, 
            score: 0.0, 
            cases: Vec::new(), 
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled,
    Default,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Result {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    CompilationError,
    #[serde(rename = "Compilation Success")]
    CompilationSuccess,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
    #[serde(rename = "System Error")]
    SystemError,
    #[serde(rename = "SPJ Error")]
    SPJError,
    Skipped,
    Default,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CaseResult {
    pub id: u64,
    pub result: Result,
    pub time: u64,
    pub memory: u64,
    pub info: String,
}