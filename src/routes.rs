use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use lazy_static::lazy_static;

use serde_json::{json, Value};
use std::{thread::sleep, time::Duration};

use tracing::{error, info};

lazy_static! {
    pub static ref DEFAULT_RESPONSE_BODY: Value = json!({
    "statusCode": 500,
    "body": {
        "message": "hello, Chaos!!!"
    }
    });
}

#[derive(Clone)]
pub struct AppState {
    pub runtime_api_address: String,
}

const ENABLE_LATENCY_ENV_NAME: &str = "CHAOS_EXTENSION__LAMBDA__ENABLE_LATENCY";
const LATENCY_PROBABILITY_ENV_NAME: &str = "CHAOS_EXTENSION__LAMBDA__LATENCY_PROBABILITY";
const LATENCY_VALUE_ENV_NAME: &str = "CHAOS_EXTENSION__LAMBDA__LATENCY_VALUE";

const ENABLE_CHANGE_REPONSE_BODY_ENV_NAME: &str =
    "CHAOS_EXTENSION__RESPONSE__ENABLE_CHANGE_REPONSE_BODY";
const REPONSE_PROBABILITY_ENV_NAME: &str = "CHAOS_EXTENSION__RESPONSE__CHANGE_RESPONSE_PROBABILITY";
const DEFAULT_RESPONSE_ENV_NAME: &str = "CHAOS_EXTENSION__RESPONSE__DEFAULT_RESPONSE";

pub async fn get_next_invocation(State(state): State<AppState>) -> impl IntoResponse {
    info!("get_next_invocation was invoked");
    let resp = reqwest::get(format!(
        "http://{}/2018-06-01/runtime/invocation/next",
        state.runtime_api_address
    ))
    .await
    .unwrap();
    let enable_timeout = str_to_bool(
        std::env::var(ENABLE_LATENCY_ENV_NAME)
            .unwrap_or("false".to_string())
            .as_str(),
        false,
    );

    let timeout_probability = std::env::var(LATENCY_PROBABILITY_ENV_NAME)
        .unwrap_or("0.9".to_string())
        .parse()
        .unwrap_or(0.9);

    let latency = std::env::var(LATENCY_VALUE_ENV_NAME)
        .unwrap_or("900".to_string())
        .parse()
        .unwrap_or(15 * 60);
    let probability = rand::random::<f64>();
    if enable_timeout {
        info!("Latency injection enabled");
        info!(
            "Chosen probability - {}, configured probability - {}",
            probability, timeout_probability
        );

        if probability < timeout_probability {
            info!("Added latency to Lambda - {} seconds", latency);
            sleep(Duration::from_secs(latency));
        }
    }

    let mut headers = resp.headers().clone();
    // Chunked respinses are causing issues.
    headers.remove("transfer-encoding");
    let status = resp.status().as_u16();
    let status = StatusCode::from_u16(status).unwrap();

    let data = resp.text().await.unwrap();

    (status, headers, data)
}

pub async fn post_invoke_response(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
    data: String,
) -> impl IntoResponse {
    info!("post_invoke_response was invoked");
    // Send the request

    let enable_change_reponse = str_to_bool(
        std::env::var(ENABLE_CHANGE_REPONSE_BODY_ENV_NAME)
            .unwrap_or("false".to_string())
            .as_str(),
        false,
    );

    let response_probability = std::env::var(REPONSE_PROBABILITY_ENV_NAME)
        .unwrap_or("0.9".to_string())
        .parse()
        .unwrap_or(0.9);

    let probability = rand::random::<f64>();

    let mut body = data;

    if enable_change_reponse {
        info!("Change response injection enabled");
        info!(
            "Chosen probability - {}, configured probability - {}",
            probability, response_probability
        );

        if probability < response_probability {
            body = std::env::var(DEFAULT_RESPONSE_ENV_NAME)
                .unwrap_or(DEFAULT_RESPONSE_BODY.to_string());
            info!("Changing response body - {}", &body);
        }
    }

    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/invocation/{}/response",
            state.runtime_api_address, request_id
        ))
        .body(body.clone())
        .send()
        .await
        .unwrap();

    let headers = resp.headers().clone();
    let status = resp.status().as_u16();
    let status = StatusCode::from_u16(status).unwrap();

    (status, headers, resp.text().await.unwrap())
}

pub async fn post_initialization_error(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    info!("post_initialization_error was invoked");
    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/init/error",
            state.runtime_api_address
        ))
        .body(body.clone())
        .headers(headers)
        .send()
        .await
        .unwrap();

    let headers = resp.headers().clone();
    let status = resp.status().as_u16();
    let status = StatusCode::from_u16(status).unwrap();

    (status, headers, body)
}

pub async fn post_invoke_error(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    info!("post_invoke_error was invoked");
    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/invocation/{}/error",
            state.runtime_api_address, request_id
        ))
        .body(body.clone())
        .headers(headers)
        .send()
        .await
        .unwrap();

    let headers = resp.headers().clone();
    let status = resp.status().as_u16();
    let status = StatusCode::from_u16(status).unwrap();

    (status, headers, body)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/2018-06-01/runtime/invocation/next",
            get(get_next_invocation),
        )
        .route(
            "/2018-06-01/runtime/invocation/:request_id/response",
            post(post_invoke_response),
        )
        .route(
            "/2018-06-01/runtime/init/error",
            post(post_initialization_error),
        )
        .route(
            "/2018-06-01/runtime/invocation/:request_id/error",
            post(post_invoke_error),
        )
        .with_state(state)
}
fn str_to_bool(input: &str, default: bool) -> bool {
    match input.to_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => {
            error!("Error: Invalid input string. Expected 'true' or 'false'.");
            default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::env;
    use std::time::Instant;

    use tower::ServiceExt;
    use wiremock::matchers::{body_string, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_next_invocation_test_added_latency() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/2018-06-01/runtime/invocation/next"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        let app = router(AppState {
            runtime_api_address: mock_server.uri().replace("http://", ""),
        });

        env::set_var(ENABLE_LATENCY_ENV_NAME, "true");
        env::set_var(LATENCY_PROBABILITY_ENV_NAME, "1.0");
        env::set_var(LATENCY_VALUE_ENV_NAME, "2");

        let start = Instant::now();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/2018-06-01/runtime/invocation/next")
                    .body(Body::from(
                        serde_json::to_vec(&json!([1, 2, 3, 4])).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let duration = start.elapsed();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(duration.as_secs() >= 2);
    }

    #[tokio::test]
    async fn post_invoke_response_test_change_reposne_default_value() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            // .and(body_json(DEFAULT_RESPONSE_BODY.to_string()))
            .and(path("/2018-06-01/runtime/invocation/1234/response"))
            .and(body_string(DEFAULT_RESPONSE_BODY.to_string()))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        let app = router(AppState {
            runtime_api_address: mock_server.uri().replace("http://", ""),
        });

        env::set_var(ENABLE_CHANGE_REPONSE_BODY_ENV_NAME, "true");
        env::set_var(REPONSE_PROBABILITY_ENV_NAME, "1.0");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/2018-06-01/runtime/invocation/1234/response")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
