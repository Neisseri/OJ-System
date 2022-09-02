use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProblemType {
    Standard,
    Strict,
    Spj,
    DynamicRanking
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Server {
    pub bind_address: Option<String>,
    pub bind_port: Option<u64>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Case {
    pub score: f64,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u64,
    pub memory_limit: u64
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Misc {

}

#[derive(Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: u64,
    pub name: String,
    pub r#type: String,
    pub misc: Option<Misc>,
    pub cases: Vec<Case>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: Server,
    pub problems: Vec<Problem>,
    pub languages: Vec<Language>
}

pub fn get_config() -> Config {
    let mut config_address: String = String::new();
    let mut read: bool = false;
    // for arg in std::env::args() { println!("{}", &arg); }
    // used to get the command arguments
    for arg in std::env::args() {
        if read == true {
            config_address = arg.clone();
            break;
        }
        if arg == "--config" || arg == "-c" {
            read = true;
        }
    } // read the address of the config file

    // println!("{}", &config_address);
    let config = {
        let json_record = std::fs::read_to_string(&config_address);
        let s: String = json_record.unwrap();
        // println!("{}", s);
        serde_json::from_str::<Config>(&s).unwrap()
    };
    config
}