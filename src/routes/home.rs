//! Module defining routes for `/`.

use axum::Extension;
use crate::outcome;


/// Handler for `GET /`.
///
/// Pings Redis to verify that everything is working correctly.
pub async fn route_home_get(
    Extension(rclient): Extension<redis::Client>
) -> outcome::RequestResult {

    log::trace!("Connecting to Redis...");
    let mut rconn = rclient.get_async_connection().await
        .map_err(outcome::redis_conn_failed)?;

    log::trace!("Sending PING and expecting PONG...");
    redis::cmd("PING")
        .query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?
        .eq("PONG")
        .then(outcome::success_null)
        .ok_or_else(outcome::redis_unexpected_behaviour)
}
