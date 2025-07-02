use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: TagDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResponse {
    pub models: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
struct TagsBackendResponse {
    models: Vec<Tag>,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub routing_table: Arc<RwLock<HashMap<String, String>>>,
    pub model_cache: Arc<RwLock<Vec<ModelInfo>>>,
    pub tag_cache: Arc<RwLock<Vec<Tag>>>,
    pub client: Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Arc::new(RwLock::new(Vec::new())),
            tag_cache: Arc::new(RwLock::new(Vec::new())),
            client: Client::new(),
        }
    }
}

pub async fn refresh_models_loop(state: AppState) {
    let mut interval = interval(Duration::from_secs(state.config.refresh_interval));

    loop {
        interval.tick().await;
        let mut model_cache = Vec::new();
        let mut tag_cache = Vec::new();
        let mut routing_table = HashMap::new();

        for backend in &state.config.backends {
            match state
                .client
                .get(format!("{}/v1/models", backend.url))
                .send()
                .await
            {
                Ok(resp) => match resp.json::<ModelsResponse>().await {
                    Ok(models_response) => {
                        for model in models_response.data {
                            routing_table.insert(model.id.clone(), backend.url.clone());
                            model_cache.push(model);
                        }
                    }
                    Err(err) => error!("Failed to parse models from {}: {}", backend.name, err),
                },
                Err(err) => error!("Failed to reach backend {}: {}", backend.name, err),
            }

            match state
                .client
                .get(format!("{}/api/tags", backend.url))
                .send()
                .await
            {
                Ok(resp) => match resp.json::<TagsBackendResponse>().await {
                    Ok(tags_response) => {
                        tag_cache.extend(tags_response.models);
                    }
                    Err(err) => error!("Failed to parse tags from {}: {}", backend.name, err),
                },
                Err(err) => error!("Failed to reach backend {} for tags: {}", backend.name, err),
            }

        }

        *state.routing_table.write().await = routing_table;
        
        let model_count = model_cache.len();
        *state.model_cache.write().await = model_cache;
        
        let tag_count = tag_cache.len();
        *state.tag_cache.write().await = tag_cache;

        info!(
            "Cache refreshed. {} models and {} tags available.",
            model_count,
            tag_count
        );
    }
}
