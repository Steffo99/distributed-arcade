use axum::extract::Json;
use axum::http::StatusCode;
use serde_json::Value;

/// The `([StatusCode], Body)` tuple returned by API handlers.
pub(crate) type RequestTuple = (StatusCode, Json<Value>);

/// A [`Result`] made of two [`RequestTuple`]s to make handling errors easier.
pub(crate) type RequestResult = Result<RequestTuple, RequestTuple>;

macro_rules! req_error {
    ( $val:tt ) => {
        Json(serde_json::json!({
            "ok": false, 
            "error": $val
        }))
    };
}

/// Macro used to build a API error.
pub(crate) use req_error;

macro_rules! req_success {
    ( $val:tt ) => {
        Json(serde_json::json!({
            "ok": true,
            "data": $val
        }))
    }
}

/// Macro used to build a API success.
pub(crate) use req_success;

/// The server could not connect to Redis.
pub(crate) fn redis_conn_failed() -> RequestTuple {
    (
        StatusCode::GATEWAY_TIMEOUT, 
        req_error!("Could not connect to Redis")
    )
}

/// The execution of a command in Redis failed.
pub(crate) fn redis_cmd_failed() -> RequestTuple {
    (
        StatusCode::BAD_GATEWAY, 
        req_error!("Could not execute Redis command")
    )
}

/// The result of a command in Redis is unexpected.
pub(crate) fn redis_unexpected_behaviour() -> RequestTuple {
    (
        StatusCode::INTERNAL_SERVER_ERROR, 
        req_error!("Redis gave an unexpected response")
    )
}

/// The request succeeded, and there's no data to be returned.
pub(crate) fn success_null() -> RequestTuple {
    (
        StatusCode::OK, 
        req_success!(null)
    )
}
