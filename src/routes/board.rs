//! Module defining routes for `/board/`.

use axum::http::StatusCode;
use axum::extract::{Extension, Json};
use redis::AsyncCommands;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::shortcuts::redis::RedisConnectOr504;
use crate::shortcuts::token::Generate;
use crate::utils::sorting::SortingOrder;
use crate::utils::kebab::Skewer;
use crate::utils::token::SecureToken;


/// Expected input data for [`POST /board/`](route_board_post).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardBody {
    /// The name of the board to create.
    pub(crate) name: String,
    /// The [`SortingOrder`] of the scores in the board to create.
    pub(crate) order: SortingOrder,
}


/// Ensure that there is nothing stored at a certain Redis key.
async fn ensure_key_is_empty(rconn: &mut redis::aio::Connection, key: &str) -> Result<(), outcome::RequestTuple> {
    log::trace!("Ensuring that the Redis key `{key}` does not contain anything...");

    redis::cmd("TYPE").arg(&key)
        .query_async::<redis::aio::Connection, String>(rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?
        .eq("none")
        .then_some(())
        .ok_or((StatusCode::CONFLICT, outcome::req_error!("Board already exists")))
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
        .map_err(|_| outcome::redis_cmd_failed())?;

    log::trace!("Setting board order...");
    rconn.set(&order_key, Into::<&str>::into(order)).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    
    log::trace!("Setting board token...");
    rconn.set(&token_key, &token.0).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    
    log::trace!("Executing Redis transaction...");
    redis::cmd("EXEC").query_async(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    Ok((
        StatusCode::CREATED,
        outcome::req_success!((token.0))
    ))
}
