//! Module defining routes for `/score/`.

use axum::http::StatusCode;
use axum::http::header::HeaderMap;
use axum::extract::{Extension, Json, Query};
use redis::AsyncCommands;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;
use crate::shortcuts::token::Authorize;
use crate::utils::kebab::Skewer;
use crate::utils::sorting::SortingOrder;


/// Query parameters for `/score/` routes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteScoreQuery {
    /// The board to access.
    pub board: String,
    /// The name of the player to access the score of.
    pub player: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteScoreResponse {
    /// The score the user has on the board.
    pub score: f64,
    /// The position of the user relative to the other users on the board, zero-based.
    pub rank: usize,
}


/// Handler for `GET /score/`.
pub(crate) async fn route_score_get(
    // Request query
    Query(RouteScoreQuery {board, player}): Query<RouteScoreQuery>,
    // Redis client
    Extension(rclient): Extension<redis::Client>,
) -> outcome::RequestResult {
    let board = board.to_kebab_lowercase();
    let player = player.to_kebab_lowercase();

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{board}:order");
    let scores_key = format!("board:{board}:scores");

    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Getting score...");
    let score = rconn.zscore(&scores_key, &player).await
        .map_err(outcome::redis_cmd_failed)?;
    log::trace!("Score is: {score:?}");

    log::trace!("Determining sorting order...");
    let order = rconn.get::<&str, String>(&order_key).await
        .map_err(outcome::redis_cmd_failed)?;
    let order = SortingOrder::try_from(order.as_str())
        .map_err(|_| outcome::redis_unexpected_behaviour())?;
    log::trace!("Sorting order is: {order:?}");

    log::trace!("Getting rank...");
    let rank = match order {
        SortingOrder::Ascending => rconn.zrank::<&str, &str, usize>(&scores_key, &player),
        SortingOrder::Descending => rconn.zrevrank::<&str, &str, usize>(&scores_key, &player),
    }.await.map_err(outcome::redis_cmd_failed)?;
    log::trace!("Rank is: {rank:?}");

    let result = RouteScoreResponse {score, rank};

    Ok((
        StatusCode::OK,
        outcome::req_success!(result)
    ))
}


/// Handler for `PUT /score/`.
pub(crate) async fn route_score_put(
    // Request headers
    headers: HeaderMap,
    // Request query
    Query(RouteScoreQuery {board, player}): Query<RouteScoreQuery>,
    // Request body
    Json(score): Json<f64>,
    // Redis client
    Extension(rclient): Extension<redis::Client>,
) -> outcome::RequestResult {
    let board = board.to_kebab_lowercase();
    let player = player.to_kebab_lowercase();

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{board}:order");
    let token_key = format!("board:{board}:token");
    let scores_key = format!("board:{board}:scores");

    let token = headers.get_authorization_or_401("Bearer")?;
    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Checking if the token exists and matches...");
    let btoken = rconn.get::<&str, String>(&token_key).await
        .map_err(outcome::redis_cmd_failed)?;

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
        .map_err(outcome::redis_cmd_failed)?;
    let order = SortingOrder::try_from(order.as_str())
        .map_err(|_| outcome::redis_unexpected_behaviour())?;
    log::trace!("Sorting order is: {order:?}");

    log::trace!("Inserting score: {score:?}");
    let changed = redis::cmd("ZADD").arg(&scores_key).arg(order.zadd_mode()).arg("CH").arg(&score).arg(&player)
        .query_async::<redis::aio::Connection, i32>(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?;

    log::trace!("Getting the new score...");
    let nscore = rconn.zscore::<&str, &str, f64>(&scores_key, &player).await
        .map_err(outcome::redis_cmd_failed)?;
    log::trace!("Received score: {nscore:?}");

    log::trace!("Getting rank...");
    let rank = match order {
        SortingOrder::Ascending => rconn.zrank::<&str, &str, usize>(&scores_key, &player),
        SortingOrder::Descending => rconn.zrevrank::<&str, &str, usize>(&scores_key, &player),
    }.await.map_err(outcome::redis_cmd_failed)?;
    log::trace!("Rank is: {rank:?}");

    let result = RouteScoreResponse {score, rank};

    Ok((
        match changed.gt(&0) {
            true => StatusCode::CREATED,
            false => StatusCode::OK,
        },
        outcome::req_success!(result)
    ))
}