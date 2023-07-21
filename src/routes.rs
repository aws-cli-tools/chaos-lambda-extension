use lazy_static::lazy_static;
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use reqwest::header::TRANSFER_ENCODING;
use serde_json::{json, Value};
use std::{thread::sleep, time::Duration};

use tracing::{error, info};

lazy_static! {
    pub static ref AWS_LAMBDA_RUNTIME_API: String =
        std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API!");
}

const ENABLE_TIMEOUT_ENV_NAME: &str = "CHAOS_EXTENSION__LAMBDA__ENABLE_TIMEOUT";
const TIMEOUT_PROBABILITY_ENV_NAME: &str = "CHAOS_EXTENSION__LAMBDA__TIMEOUT_PROBABILITY";
const ENABLE_CHANGE_REPONSE_BODY_ENV_NAME: &str =
    "CHAOS_EXTENSION__RESPONSE__ENABLE_CHANGE_REPONSE_BODY";
const REPONSE_PROBABILITY_ENV_NAME: &str = "CHAOS_EXTENSION__RESPONSE__CHANGE_RESPONSE_PROBABILITY";
const DEFAULT_RESPONSE_ENV_NAME: &str = "CHAOS_EXTENSION__RESPONSE__DEFAULT_RESPONSE";

pub async fn get_next_invocation() -> Json<Value> {
    info!("get_next_invocation was invoked");
    let resp = reqwest::get(format!(
        "http://{}/2018-06-01/runtime/invocation/next",
        *AWS_LAMBDA_RUNTIME_API
    ))
    .await
    .unwrap();
    let enable_timeout = str_to_bool(
        std::env::var(ENABLE_TIMEOUT_ENV_NAME)
            .unwrap_or("false".to_string())
            .as_str(),
        false,
    );

    let timeout_probability = std::env::var(TIMEOUT_PROBABILITY_ENV_NAME)
        .unwrap_or("0.9".to_string())
        .parse()
        .unwrap_or(0.9);

    if enable_timeout && rand::random::<f64>() > timeout_probability {
        sleep(Duration::from_secs(15 * 60));
    }

    let mut headers = resp.headers().clone();
    headers.remove(TRANSFER_ENCODING);

    let data = resp.text().await.unwrap();

    let data_json: serde_json::Value = serde_json::from_str(&data).unwrap();
    Json(data_json)
}

pub async fn post_invoke_response(Path(request_id): Path<String>, data: String) -> Json<Value> {
    info!("post_invoke_response was invoked");
    // Send the request
    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/invocation/{}/response",
            *AWS_LAMBDA_RUNTIME_API, request_id
        ))
        .body(data)
        .send()
        .await
        .unwrap();

    let enable_change_reponse = str_to_bool(
        std::env::var(ENABLE_CHANGE_REPONSE_BODY_ENV_NAME)
            .unwrap_or("false".to_string())
            .as_str(),
        false,
    );

    let timeout_probability = std::env::var(REPONSE_PROBABILITY_ENV_NAME)
        .unwrap_or("0.9".to_string())
        .parse()
        .unwrap_or(0.9);

    if enable_change_reponse && rand::random::<f64>() > timeout_probability {
        let default_response = json!({
        "statusCode": 500,
        "body": {
            "message": "hello, Chaos!!!"
        }
        });
        let response_data =
            std::env::var(DEFAULT_RESPONSE_ENV_NAME).unwrap_or(default_response.to_string());
        Json(serde_json::from_str(&response_data).unwrap_or(default_response))
    } else {
        let data = resp.text().await.unwrap();
        let data_json: serde_json::Value = serde_json::from_str(&data).unwrap();
        Json(data_json)
    }
}

pub async fn post_initialization_error(headers: HeaderMap, body: String) -> impl IntoResponse {
    info!("post_initialization_error was invoked");
    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/init/error",
            *AWS_LAMBDA_RUNTIME_API
        ))
        .json(&body.as_str())
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
    Path(request_id): Path<String>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    info!("post_invoke_error was invoked");
    let resp = reqwest::Client::new()
        .post(format!(
            "http://{}/2018-06-01/runtime/init/error",
            *AWS_LAMBDA_RUNTIME_API
        ))
        .json(&body.as_str())
        .headers(headers)
        .send()
        .await
        .unwrap();

    let headers = resp.headers().clone();
    let status = resp.status().as_u16();
    let status = StatusCode::from_u16(status).unwrap();

    (status, headers, body)
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
