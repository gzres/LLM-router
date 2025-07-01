use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
};
use llm_router::{
    config::{AuthConfig, BackendConfig, Config},
    model::AppState,
    router,
};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_full_request_flow() {
    // Setup mock LLM backend
    let mock_server = MockServer::start().await;

    // Setup model list endpoint
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "gpt-3.5-turbo", "type": "llm"}]
        })))
        .mount(&mock_server)
        .await;

    // Setup completions endpoint
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "Test response"}}]
        })))
        .mount(&mock_server)
        .await;

    // Create test configuration
    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test-backend".to_string(),
            url: mock_server.uri(),
            auth: Some(AuthConfig::Bearer {
                token: "test-token".to_string(),
            }),
        }],
    };

    let state = AppState::new(config);

    // Manually update routing table for test
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("gpt-3.5-turbo".to_string(), mock_server.uri());
    }

    // Make a completion request
    let response = router::forward_request(
        axum::extract::State(state),
        HeaderMap::new(),
        Body::from(
            json!({
                "model": "gpt-3.5-turbo",
                "messages": [{"role": "user", "content": "Hello"}]
            })
            .to_string(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
}
