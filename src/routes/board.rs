//! Module defining routes for `/board/`.

use axum::http::StatusCode;
use axum::extract::{Extension, Json, Query};
use redis::AsyncCommands;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;
use crate::shortcuts::token::Generate;
use crate::utils::sorting::SortingOrder;
use crate::utils::kebab::Skewer;
use crate::utils::token::SecureToken;


/// Expected body for [`POST /board/`](route_board_post).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardBody {
    /// The name of the board to create.
    pub(crate) name: String,
    /// The [`SortingOrder`] of the scores in the board to create.
    pub(crate) order: SortingOrder,
}


/// Expected query params for [`GET /board/`](route_board_get).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardQuery {
    /// The name of the board to access.
    pub(crate) board: String,
    /// The offset to start returning scores from.
    pub(crate) offset: usize,
    /// How many scores to return.
    pub(crate) size: usize,
}


/// Expected response for [`GET /board/`](route_board_get).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardResponse {
    /// The offset of the next page.
    pub(crate) offset: usize,
    /// The scores of the current page.
    pub(crate) scores: Vec<ScoreObject>,
}


/// A score set by a player, as a serializable struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ScoreObject {
    /// The name of the player who set the score.
    pub(crate) name: String,
    /// The score that the player set.
    pub(crate) score: f64,
}

impl From<(String, f64)> for ScoreObject {
    fn from(t: (String, f64)) -> Self {
        ScoreObject {name: t.0, score: t.1}
    }
}

impl From<(usize, Vec<(String, f64)>)> for RouteBoardResponse {
    fn from(t: (usize, Vec<(String, f64)>)) -> Self {
        RouteBoardResponse {
            offset: t.0,
            scores: t.1.into_iter().map(From::from).collect()
        }
    }
}

/// Ensure that there is nothing stored at a certain Redis key.
async fn ensure_key_is_empty(rconn: &mut redis::aio::Connection, key: &str) -> Result<(), outcome::RequestTuple> {
    log::trace!("Ensuring that the Redis key `{key}` does not contain anything...");

    redis::cmd("TYPE").arg(&key)
        .query_async::<redis::aio::Connection, String>(rconn).await
        .map_err(outcome::redis_cmd_failed)?
        .eq("none")
        .then_some(())
        .ok_or((StatusCode::CONFLICT, outcome::req_error!("Board already exists")))
}

/// Handler for `GET /board/`.
pub(crate) async fn route_board_get(
    // Request query
    Query(RouteBoardQuery {board, offset, size}): Query<RouteBoardQuery>,
    // Redis client
    Extension(rclient): Extension<redis::Client>,
) -> outcome::RequestResult {

    let board = board.to_kebab_lowercase();

    log::trace!("Ensuring the size is within limits...");
    if size > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            outcome::req_error!("Cannot request more than 500 scores at a time")
        ))
    }

    log::trace!("Determining the Redis key name...");
    let scores_key = format!("board:{board}:scores");

    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Retrieving scores from {board}...");
    let result: RouteBoardResponse = redis::cmd("ZSCAN").arg(&scores_key).arg(offset).arg("COUNT").arg(&size)
        .query_async::<redis::aio::Connection, (usize, Vec<(String, f64)>)>(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?
        .into();

    Ok((StatusCode::OK, outcome::req_success!(result)))
}


/// Handler for `POST /board/`.
///
/// Creates a new board, storing the details on [Redis].
///
/// Will refuse to overwrite an already existing board.
/// 
/// Be aware that once created, boards cannot be deleted, if not manually via `redis-cli`.
/// 
/// If successful, returns [`StatusCode::CREATED`].
pub(crate) async fn route_board_post(
    Extension(rclient): Extension<redis::Client>,
    Json(RouteBoardBody {name, order}): Json<RouteBoardBody>,
) -> outcome::RequestResult {

    let name = name.to_kebab_lowercase();

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{name}:order");
    let token_key = format!("board:{name}:token");
    let scores_key = format!("board:{name}:scores");

    let mut rconn = rclient.get_connection_or_504().await?;

    log::trace!("Ensuring a board does not already exist...");
    ensure_key_is_empty(&mut rconn, &order_key).await?;
    ensure_key_is_empty(&mut rconn, &token_key).await?;
    ensure_key_is_empty(&mut rconn, &scores_key).await?;

    let token = SecureToken::new_or_500()?;

    log::debug!("Creating board: {name:?}");

    log::trace!("Starting Redis transaction...");
    redis::cmd("MULTI").query_async(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?;

    log::trace!("Setting board order...");
    rconn.set(&order_key, Into::<&str>::into(order)).await
        .map_err(outcome::redis_cmd_failed)?;
    
    log::trace!("Setting board token...");
    rconn.set(&token_key, &token.0).await
        .map_err(outcome::redis_cmd_failed)?;
    
    log::trace!("Executing Redis transaction...");
    redis::cmd("EXEC").query_async(&mut rconn).await
        .map_err(outcome::redis_cmd_failed)?;

    Ok((
        StatusCode::CREATED,
        outcome::req_success!((token.0))
    ))
}
