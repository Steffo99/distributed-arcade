//! Module defining routes for `/`.

use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;


/// Possible results for `GET /`.
pub enum OutcomeHomeGet {
    /// Could not connect to Redis.
    RedisConnectionError,
    /// Could not PING Redis.
    RedisPingError,
    /// Did not get a PONG back from Redis.
    RedisPongError,
    /// Ping successful.
    Success,
}

use OutcomeHomeGet::*;

impl IntoResponse for OutcomeHomeGet {
    fn into_response(self) -> Response {
        let (status, response) = match self {
            RedisConnectionError => (StatusCode::GATEWAY_TIMEOUT, json!("Could not connect to Redis")),
            RedisPingError => (StatusCode::BAD_GATEWAY, json!("Could not ping Redis")),
            RedisPongError => (StatusCode::INTERNAL_SERVER_ERROR, json!("Redis did not pong back")),
            Success => (StatusCode::OK, json!("Welcome to distributed_arcade! Redis seems to be working correctly."))
        };

        IntoResponse::into_response((status, Json(response)))
    }
}


/// Handler for `GET /`.
///
/// Pings Redis to verify that everything is working correctly.
pub async fn route_home_get(
    Extension(rclient): Extension<redis::Client>
) -> Result<OutcomeHomeGet, OutcomeHomeGet> {

    log::trace!("Connecting to Redis...");
    let mut rconn = rclient.get_async_connection().await
        .map_err(|_| RedisConnectionError)?;

    log::trace!("Sending PING...");
    let pong = redis::cmd("PING").query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(|_| RedisPingError)?;

    log::trace!("Expecting PONG: {pong:?}");
    pong.eq("PONG")
        .then_some(Success).ok_or(RedisPongError)
}
