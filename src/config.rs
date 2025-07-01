use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub refresh_interval: u64,
    pub backends: Vec<BackendConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub name: String,
    pub url: String,
}

pub fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path).expect("Failed to read config");
    serde_yml::from_str(&content).expect("Failed to parse config")
}
