//! Module defining routes for `/score/`.

use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::extract::{Extension, Json};
use redis::AsyncCommands;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;
use crate::shortcuts::token::Authorize;
use crate::utils::kebab::Skewer;
use crate::utils::sorting::SortingOrder;


/// Expected input data for `PUT /score/`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteScorePutInput {
    /// The board to submit the score to.
    pub board: String,
    /// The score to submit.
    pub score: f64,
    /// The name of the player submitting the score.
    pub player: String,
}


/// Handler for `PUT /score/`.
pub(crate) async fn route_score_put(
    // Request headers
    headers: HeaderMap,
    // Request body
    Json(RouteScorePutInput {board, score, player}): Json<RouteScorePutInput>,
    // Redis client
    Extension(rclient): Extension<redis::Client>,
) -> outcome::RequestResult {
    let board = board.to_kebab_lowercase();

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{board}:order");
    let token_key = format!("board:{board}:token");
    let scores_key = format!("board:{board}:scores");

    let token = headers.get_authorization_or_401("X-Board-Token")?;
    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Checking if the token exists and matches...");
    let btoken = rconn.get::<&str, String>(&token_key).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    if btoken.is_empty() {
        log::trace!("Token is not set, board does not exist...");
        return Err((StatusCode::NOT_FOUND, outcome::req_error!("No such board")))
    }

    if btoken != token {
        log::trace!("Token does not match, forbidding...");
        return Err((StatusCode::FORBIDDEN, outcome::req_error!("Invalid board token"))) 
    }
    
    log::trace!("Determining sorting order...");
    let order = rconn.get::<&str, String>(&order_key).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    let order = SortingOrder::try_from(order.as_str())
        .map_err(|_| outcome::redis_unexpected_behaviour())?;
    log::trace!("Sorting order is: {order:?}");

    log::trace!("Inserting score: {score:?}");
    redis::cmd("ZADD").arg(&scores_key).arg(order.zadd_mode()).arg(&score).arg(&player).query_async(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    log::trace!("Getting the new score...");
    let nscore = rconn.zscore::<&str, &str, f64>(&scores_key, &player).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    log::trace!("Received score: {nscore:?}");
    
    Ok((
        StatusCode::OK,
        outcome::req_success!(nscore)
    ))
}