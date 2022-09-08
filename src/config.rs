use serde::{Serialize, Deserialize};
use crate::persistent_storage::{clear_persistent_storage, 
    read_persistent_storage};
use std::io::Error;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProblemType {
    Standard,
    Strict,
    Spj,
    DynamicRanking
} // The Judge Method for the problem

#[derive(Clone, Serialize, Deserialize)]
pub struct Server {
    pub bind_address: Option<String>,
    pub bind_port: Option<u64>
} // The Server's bind address and port

#[derive(Clone, Serialize, Deserialize)]
pub struct Case {
    pub score: f64,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u64,
    pub memory_limit: u64
} // the information of test cases

#[derive(Clone, Serialize, Deserialize)]
pub struct Misc {
    pub packing: Option<Vec<Vec<usize>>>,
    pub special_judge: Option<Vec<String>>,
    pub dynamic_ranking_ratio: Option<f64>,
} // the additional information for advanced requirements

#[derive(Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: usize,
    pub name: String,
    pub r#type: String,
    pub misc: Option<Misc>,
    pub cases: Vec<Case>,
} // the description for problems

#[derive(Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>
} // the information of the avilable languages

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: Server,
    pub problems: Vec<Problem>,
    pub languages: Vec<Language>
} // config json

pub fn get_config() -> Config {

    let mut config_address: String = String::new();
    let mut read_config: bool = false;
    let mut read_storage: bool = true;
    // if need to read json file to get the persistent storage

    // used to get the command arguments
    for arg in std::env::args() {
        if read_config == true {
            config_address = arg.clone();
            read_config = false;
        }
        if arg == "--config" || arg == "-c" {
            read_config = true;
        }
        if arg == "--flush-data" {
            clear_persistent_storage();
            read_storage = false; //clear the storage and don't need to read
        }
    } // read the address of the config file and other arguments

    let config: Config = {
        let json_record: Result<String, Error> = 
            std::fs::read_to_string(&config_address);
        let s: String = json_record.unwrap();
        serde_json::from_str::<Config>(&s).unwrap()
    }; // read the file to `Config` json struct
    if read_storage == true {
        read_persistent_storage();
    }
    config
}