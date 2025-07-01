use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::interval};
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub routing_table: Arc<RwLock<HashMap<String, String>>>,
    pub model_cache: Arc<RwLock<Vec<ModelInfo>>>,
    pub client: Client,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Arc::new(RwLock::new(Vec::new())),
            client: Client::new(),
        }
    }
}

pub async fn refresh_models_loop(state: AppState) {
    let mut interval = interval(Duration::from_secs(state.config.refresh_interval));

    loop {
        interval.tick().await;
        let mut model_cache = Vec::new();
        let mut routing_table = HashMap::new();

        for backend in &state.config.backends {
            match state
                .client
                .get(format!("{}/v1/models", backend.url))
                .send()
                .await
            {
                Ok(resp) => match resp.json::<HashMap<String, Vec<ModelInfo>>>().await {
                    Ok(json) => {
                        for model in json.get("data").cloned().unwrap_or_default() {
                            routing_table.insert(model.id.clone(), backend.url.clone());
                            model_cache.push(model);
                        }
                    }
                    Err(err) => error!("Failed to parse models from {}: {}", backend.name, err),
                },
                Err(err) => error!("Failed to reach backend {}: {}", backend.name, err),
            }
        }

        *state.routing_table.write().await = routing_table;
        *state.model_cache.write().await = model_cache;
        info!("Model routing table refreshed.");
    }
}
