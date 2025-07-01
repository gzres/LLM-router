#[cfg(test)]
mod tests {
    use llm_router::{
        config::{BackendConfig, Config},
        model::{AppState, refresh_models_loop},
    };
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_app_state_creation() {
        let config = Config {
            refresh_interval: 300,
            backends: vec![BackendConfig {
                name: "test".to_string(),
                url: "http://localhost:8000".to_string(),
                auth: None,
            }],
        };

        let state = AppState::new(config);
        assert_eq!(state.config.refresh_interval, 300);
    }

    #[tokio::test]
    async fn test_refresh_models_loop() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "model-1", "type": "llm"},
                    {"id": "model-2", "type": "llm"}
                ]
            })))
            .expect(1..)
            .mount(&mock_server)
            .await;

        let config = Config {
            refresh_interval: 1,
            backends: vec![BackendConfig {
                name: "test".to_string(),
                url: mock_server.uri(),
                auth: None,
            }],
        };

        let state = AppState::new(config);
        let state_clone = state.clone();

        let handle = tokio::spawn(async move {
            refresh_models_loop(state).await;
        });

        tokio::time::sleep(Duration::from_secs(2)).await;

        let cache = state_clone.model_cache.read().await;
        assert!(cache.len() >= 2);
        assert!(cache.iter().any(|m| m.id == "model-1"));
        assert!(cache.iter().any(|m| m.id == "model-2"));

        let routing = state_clone.routing_table.read().await;
        assert!(routing.contains_key("model-1"));
        assert!(routing.contains_key("model-2"));

        handle.abort();
    }
}
