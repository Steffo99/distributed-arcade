//! Module defining routes for `/`.

use axum::Extension;
use axum::http::StatusCode;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;


/// Handler for `GET /`.
pub(crate) async fn route_home_get() -> StatusCode {
    log::trace!("Echoing back a success...");
    StatusCode::NO_CONTENT
}


/// Handler for `POST /`.
pub(crate) async fn route_home_post(
    Extension(rclient): Extension<redis::Client>
) -> Result<StatusCode, outcome::RequestTuple> {

    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Sending PING and expecting PONG...");
    redis::cmd("PING")
        .query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?
        .eq("PONG")
        .then(|| StatusCode::NO_CONTENT)
        .ok_or_else(outcome::redis_unexpected_behaviour)
}
