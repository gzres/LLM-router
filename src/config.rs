use serde::Deserialize;
use std::fs;
use std::path::Path;

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

pub fn try_load_config<P: AsRef<Path>>(path: P) -> Option<Config> {
    let path = path.as_ref();

    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        if extension != "yml" && extension != "yaml" {
            return None;
        }
    }

    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_yml::from_str(&contents).ok())
}

pub fn load_config(filename: &str) -> Config {
    if let Some(config) = try_load_config(filename) {
        return config;
    }

    let extensions = ["yml", "yaml"];

    if let Some(stem) = Path::new(filename).file_stem().and_then(|s| s.to_str()) {
        for ext in extensions {
            let path = format!("{}.{}", stem, ext);
            if let Some(config) = try_load_config(&path) {
                return config;
            }
        }
    }

    panic!("Configuration file not found. Check if config.yml or config.yaml file exists");
}
