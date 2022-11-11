//! Module defining routes for `/score/`.

use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::extract::{Extension, Json};
use redis::AsyncCommands;
use regex::Regex;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::types::SortingOrder;


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


/// Expected output data for `PUT /score/`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteScorePutOutput {
    /// The best score of the player.
    score: f64,
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
    lazy_static::lazy_static! {
        static ref AUTH_HEADER_REGEX: Regex = Regex::new(r#"X-Board (\S+)"#)
            .expect("AUTH_HEADER_REGEX to be valid");
    }

    log::trace!("Checking the Authorization header...");
    let token = headers.get("Authorization")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, outcome::req_error!("Missing Authorization header")))?;

    let token = token.to_str()
        .map_err(|_| (StatusCode::BAD_REQUEST, outcome::req_error!("Malformed Authorization header")))?;

    let token = AUTH_HEADER_REGEX.captures(token)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, outcome::req_error!("Malformed Authorization header")))?;

    let token = token.get(1)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, outcome::req_error!("Invalid Authorization header")))?;

    let token = token.as_str();
    log::trace!("Received token: {token:?}");

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{board}:order");
    let token_key = format!("board:{board}:token");
    let scores_key = format!("board:{board}:scores");

    log::trace!("Connecting to Redis...");
    let mut rconn = rclient.get_async_connection().await
        .map_err(|_| outcome::redis_conn_failed())?;

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
    
    log::trace!("Determining score insertion mode...");
    let order = rconn.get::<&str, String>(&order_key).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    let order = SortingOrder::try_from(order)
        .map_err(|_| outcome::redis_unexpected_behaviour())?;

    log::trace!("Inserting score: {score:?}");
    redis::cmd("ZADD").arg(&scores_key).arg(order.zadd_mode()).arg(&score).arg(&player).query_async(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    log::trace!("Getting the new score...");
    let nscore = rconn.zscore::<&str, &str, f64>(&scores_key, &player).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    log::trace!("Received score: {nscore:?}");
    
    Ok((
        StatusCode::OK,
        Json(serde_json::to_value(RouteScorePutOutput {score: nscore}).expect("to be able to serialize RouteScorePutOutput"))
    ))
}