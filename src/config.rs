use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

lazy_static! {
    static ref CONFIG: Arc<Config> = match Config::new() {
        Ok(val) => Arc::new(val),
        Err(e) => panic!(format!("Config initialization error: {}", e.to_string()))
    };
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Version {
    pub name: String,
    pub version: String
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Exec {
    pub timeout: u64,
    pub timeout_duration: Duration
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub version: Version,
    pub exec: Exec
}

impl Config {
    pub fn new() -> Result<Config, String> {
        let mut file = match File::open("version.json") {
            Ok(file) => file,
            Err(e) => return Err(format!("Could not open version.json. Err: {}", e.to_string()))
        };

        let mut json_str = String::new();
        match file.read_to_string(&mut json_str) {
            Ok(_) => (),
            Err(e) => return Err(format!("Could not read version.json. Err: {}", e.to_string()))
        };

        let mut version: Version = match serde_json::from_str(&json_str) {
            Ok(config) => config,
            Err(e) => return Err(format!("version.json is not in json format. Err: {}", e.to_string()))
        };

        let timeout: u64 = match std::env::var_os("TIMEOUT_S") {
            Some(val) => val.to_str().unwrap().parse::<u64>().unwrap(),
            None => 10
        };

        let exec = Exec{
            timeout: timeout,
            timeout_duration: Duration::from_secs(timeout)
        };

        version.version = match std::env::var_os("TAG") {
            Some(val) => String::from(val.to_str().unwrap()),
            None => version.version
        };

        let config: Config = Config{
            version: version,
            exec: exec
        };

        Ok(config)
    }

    pub fn get_current<'a>() -> &'a Config {
        &CONFIG
    }
}