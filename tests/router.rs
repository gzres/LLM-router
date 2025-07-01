use axum::extract::State;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
};
use base64::Engine;
use http_body_util::BodyExt;
use llm_router::{
    ModelInfo,
    config::{AuthConfig, BackendConfig, Config},
    model::AppState,
    router::{forward_completion, forward_request, healthz, list_models},
};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_test_app(mock_server_url: String) -> Router {
    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server_url.clone(), // Klonujemy tu
            auth: Some(AuthConfig::Bearer {
                token: "test-token".to_string(),
            }),
        }],
    };

    let state = AppState::new(config);
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("test-model".to_string(), mock_server_url);
    }

    Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(forward_request))
        .route("/v1/completions", post(forward_completion))
        .route("/healthz", get(healthz))
        .with_state(state)
}

#[tokio::test]
async fn test_forward_request_with_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"text": "Test response"}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let app = setup_test_app(mock_server.uri()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    mock_server.verify().await;

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["choices"][0]["text"], "Test response");
}

#[tokio::test]
async fn test_list_models() {
    let mock_server = MockServer::start().await;

    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server.uri(),
            auth: None,
        }],
    };

    let state = AppState::new(config);

    let test_models = vec![
        ModelInfo {
            id: "model-1".to_string(),
            extra: Default::default(),
        },
        ModelInfo {
            id: "model-2".to_string(),
            extra: Default::default(),
        },
    ];

    {
        let mut cache = state.model_cache.write().await;
        *cache = test_models;
    }

    let response = list_models(State(state.clone())).await;

    let models = response.0.get("data").expect("Should have 'data' field");
    assert_eq!(models.len(), 2);
    assert!(models.iter().any(|m| m.id == "model-1"));
    assert!(models.iter().any(|m| m.id == "model-2"));
}

#[tokio::test]
async fn test_forward_completion() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/completions"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"text": "Test completion"}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let app = setup_test_app(mock_server.uri()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "prompt": "Complete this"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(body["choices"][0]["text"], "Test completion");
}

#[tokio::test]
async fn test_forward_with_basic_auth() {
    let mock_server = MockServer::start().await;

    let credentials = base64::engine::general_purpose::STANDARD.encode("testuser:testpass");
    let auth_header = format!("Basic {}", credentials);

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", &auth_header))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "Test response"}}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server.uri(),
            auth: Some(AuthConfig::Basic {
                username: "testuser".to_string(),
                password: "testpass".to_string(),
            }),
        }],
    };

    let state = AppState::new(config);
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("test-model".to_string(), mock_server.uri());
    }

    let app = Router::new()
        .route("/v1/chat/completions", post(forward_request))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_forward_with_custom_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("X-Custom-Auth", "custom-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "Test response"}}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server.uri(),
            auth: Some(AuthConfig::CustomHeader {
                name: "X-Custom-Auth".to_string(),
                value: "custom-token".to_string(),
            }),
        }],
    };

    let state = AppState::new(config);
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("test-model".to_string(), mock_server.uri());
    }

    let app = Router::new()
        .route("/v1/chat/completions", post(forward_request))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_forward_unknown_model() {
    let mock_server = MockServer::start().await;
    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server.uri(),
            auth: None,
        }],
    };

    let state = AppState::new(config);
    let app = Router::new()
        .route("/v1/chat/completions", post(forward_request))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "nonexistent-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body_bytes[..], b"Unknown model");
}

#[tokio::test]
async fn test_forward_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: mock_server.uri(),
            auth: None,
        }],
    };

    let state = AppState::new(config);
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("test-model".to_string(), mock_server.uri());
    }

    let app = Router::new()
        .route("/v1/chat/completions", post(forward_request))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_forward_network_error() {
    let config = Config {
        refresh_interval: 300,
        backends: vec![BackendConfig {
            name: "test".to_string(),
            url: "http://localhost:1".to_string(),
            auth: None,
        }],
    };

    let state = AppState::new(config);
    {
        let mut routing_table = state.routing_table.write().await;
        routing_table.insert("test-model".to_string(), "http://localhost:1".to_string());
    }

    let app = Router::new()
        .route("/v1/chat/completions", post(forward_request))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "model": "test-model",
                        "messages": [{"role": "user", "content": "Hello"}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "Internal forwarding error");
}

#[tokio::test]
async fn test_healthz() {
    let app = Router::new().route("/healthz", get(healthz));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body_str, "OK");
}
