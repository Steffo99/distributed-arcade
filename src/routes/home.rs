//! Module defining routes for `/`.

use axum::Extension;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;


/// Handler for `GET /`.
///
/// Verifies that the web server is working correctly.
pub(crate) async fn route_home_get() -> outcome::RequestResult {
    log::trace!("Echoing back a success...");
    Ok(outcome::success_null())
}


/// Handler for `PATCH /`.
///
/// Pings Redis to verify that everything is working correctly.
pub(crate) async fn route_home_patch(
    Extension(rclient): Extension<redis::Client>
) -> outcome::RequestResult {

    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Sending PING and expecting PONG...");
    redis::cmd("PING")
        .query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?
        .eq("PONG")
        .then(outcome::success_null)
        .ok_or_else(outcome::redis_unexpected_behaviour)
}
