use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use influxdb::Client;
use polars::error::PolarsError;
use redis::aio::ConnectionManager;
use redis::RedisError;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use serde_json::Value;
use snowflake_me::Error as SnowflakeError;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum JobEvent {
    Processing,
    Completed { result: Value },
    Failed { error: String },
}

// New AssetClass enum for validation
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AssetClass {
    Crypto,
    Forex,
}

impl fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AssetClass::Crypto => write!(f, "crypto"),
            AssetClass::Forex => write!(f, "forex"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Client,
    pub redis: ConnectionManager,
}

#[derive(Debug)]
pub enum ApiError {
    Influx(influxdb::Error),
    Polars(PolarsError),
    Json(SerdeJsonError),
    Snowflake(SnowflakeError),
    Redis(RedisError),
    BadRequest(String),
}

impl From<influxdb::Error> for ApiError {
    fn from(err: influxdb::Error) -> Self {
        ApiError::Influx(err)
    }
}

impl From<PolarsError> for ApiError {
    fn from(err: PolarsError) -> Self {
        ApiError::Polars(err)
    }
}

impl From<SerdeJsonError> for ApiError {
    fn from(err: SerdeJsonError) -> Self {
        ApiError::Json(err)
    }
}

impl From<SnowflakeError> for ApiError {
    // Added this block
    fn from(err: SnowflakeError) -> Self {
        ApiError::Snowflake(err)
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::BadRequest(err)
    }
}

impl From<RedisError> for ApiError {
    fn from(err: RedisError) -> Self {
        ApiError::Redis(err)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Influx(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("InfluxDB error: {}", err),
            ),
            ApiError::Polars(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Polars error: {}", err),
            ),
            ApiError::Json(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON serialization/deserialization error: {}", err),
            ),
            ApiError::Snowflake(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Snowflake error: {}", err),
            ),
            ApiError::Redis(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis error: {}", err),
            ),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, format!("Bad request: {}", msg)),
        };
        (status, error_message).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Influx(err) => write!(f, "InfluxDB error: {}", err),
            ApiError::Polars(err) => write!(f, "Polars error: {}", err),
            ApiError::Json(err) => write!(f, "JSON error: {}", err),
            ApiError::Snowflake(err) => write!(f, "Snowflake error: {}", err),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Redis(err) => write!(f, "Redis error: {}", err),
        }
    }
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    pub asset_class: AssetClass,
    pub tickers: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResponse {
    pub job_id: String,
    pub asset_class: String,
    pub data: Vec<Value>,
}
