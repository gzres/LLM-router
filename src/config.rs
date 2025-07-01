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
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AuthConfig {
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "basic")]
    Basic { username: String, password: String },
    #[serde(rename = "header")]
    CustomHeader { name: String, value: String },
}

pub fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path).expect("Failed to read config");
    serde_yml::from_str(&content).expect("Failed to parse config")
}