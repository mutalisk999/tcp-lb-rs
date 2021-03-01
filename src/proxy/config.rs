// #[macro_use]
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;

const CONFIG_FILE_NAME: &'static str = "lb-config.json";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub lb_log: LogConfig,
    pub lb_node: NodeConfig,
    pub lb_targets: Vec<TargetConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub log_set_level: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub listen: String,
    pub max_conn: u32,
    pub timeout: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetConfig {
    pub target_endpoint: String,
    pub target_max_conn: u32,
    pub target_timeout: u32,
}

pub fn read_config() -> Config {
    let mut config_file = File::open(CONFIG_FILE_NAME).expect("Config file not found");
    let mut json_str = String::new();
    config_file.read_to_string(&mut json_str).expect("Failure while reading config file");
    let config: Config = serde_json::from_str(&json_str).expect("Deserialize json config fail");
    config
}

#[test]
fn test_read_config() {
    let config = read_config();
    println!("config: {:?}", config);
}
