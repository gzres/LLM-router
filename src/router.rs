use crate::model::{AppState, ModelInfo};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use http_body_util::BodyExt;
use serde_json::Value;
use std::collections::HashMap;
use tracing::error;

pub async fn list_models(
    State(state): State<AppState>,
) -> Json<HashMap<&'static str, Vec<ModelInfo>>> {
    let models = state.model_cache.read().await.clone();
    Json(HashMap::from([("data", models)]))
}

pub async fn forward_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    req_body: Body,
) -> Response<Body> {
    forward(state, headers, req_body, "/v1/chat/completions").await
}

pub async fn forward_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    req_body: Body,
) -> Response<Body> {
    forward(state, headers, req_body, "/v1/completions").await
}

async fn forward(
    state: AppState,
    headers: HeaderMap,
    req_body: Body,
    endpoint: &str,
) -> Response<Body> {
    let collected = req_body.collect().await.unwrap_or_default();
    let body_bytes = collected.to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap_or_default();
    let model = json.get("model").and_then(|v| v.as_str()).unwrap_or("");

    let routing_table = state.routing_table.read().await;
    if let Some(backend_url) = routing_table.get(model) {
        let url = format!("{}{}", backend_url, endpoint);
        match state
            .client
            .post(url)
            .headers(headers.clone())
            .body(body_bytes)
            .send()
            .await
        {
            Ok(response) => {
                let mut builder = Response::builder().status(response.status());
                for (k, v) in response.headers() {
                    builder = builder.header(k, v);
                }
                let bytes = response.bytes().await.unwrap_or_default();
                builder.body(Body::from(bytes)).unwrap()
            }
            Err(err) => {
                error!("Forwarding failed: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal forwarding error",
                )
                    .into_response()
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, "Unknown model").into_response()
    }
}

pub async fn healthz() -> &'static str {
    "OK"
}
