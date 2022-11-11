//! Module defining routes for `/board/`.

use axum::http::StatusCode;
use axum::extract::{Extension, Json};
use redis::AsyncCommands;
use serde::Serialize;
use serde::Deserialize;
use crate::outcome;
use crate::types::SortingOrder;


/// Expected input data for [`POST /board/`](route_board_post).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardPostInput {
    /// The name of the board to create.
    pub(crate) name: String,
    /// The [`SortingOrder`] of the scores in the board to create.
    pub(crate) order: SortingOrder,
}

/// Expected output data for [`POST /board/`](route_board_post).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct RouteBoardPostOutput {
    /// The token to use to submit scores to the board.
    /// 
    /// ### It's a secret
    /// 
    /// Be careful to keep this visible only to the board admins!
    pub(crate) token: String,
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

/// Alphabet for base-62 encoding.
const TOKEN_CHARS: &[char; 62] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
];

/// Generate a cryptographically secure Base-62 token via [`rand::rngs::OsRng`].
fn generate_secure_token() -> Result<String, outcome::RequestTuple> {
    log::trace!("Generating a board token...");

    let mut rng = rand::rngs::OsRng::default();
    let mut token: [u32; 16] = [0; 16];

    rand::Fill::try_fill(&mut token, &mut rng)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, outcome::req_error!("Failed to generate a secure board token.")))?;
    
    Ok(
        // FIXME: only works on platforms where usize >= 32-bit?
        token.iter().map(|e| TOKEN_CHARS.get(*e as usize % 62).expect("randomly generated value to be a valid index"))
        .collect::<String>()
    )
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
    Json(RouteBoardPostInput {name, order}): Json<RouteBoardPostInput>,
) -> outcome::RequestResult {

    log::trace!("Determining the Redis key names...");
    let order_key = format!("board:{name}:order");
    let token_key = format!("board:{name}:token");
    let scores_key = format!("board:{name}:scores");

    log::trace!("Connecting to Redis...");
    let mut rconn = rclient.get_async_connection().await
        .map_err(|_| outcome::redis_conn_failed())?;

    log::trace!("Ensuring a board does not already exist...");
    ensure_key_is_empty(&mut rconn, &order_key).await?;
    ensure_key_is_empty(&mut rconn, &token_key).await?;
    ensure_key_is_empty(&mut rconn, &scores_key).await?;

    log::debug!("Creating board: {name:?}");

    let token = generate_secure_token()?;
    log::trace!("Board token is: {token:?}");

    log::trace!("Board order is: {order:?}");

    log::trace!("Starting Redis transaction...");
    redis::cmd("MULTI").query_async(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    log::trace!("Setting board order...");
    rconn.set(&order_key, String::from(order)).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    
    log::trace!("Setting board token...");
    rconn.set(&token_key, &token).await
        .map_err(|_| outcome::redis_cmd_failed())?;
    
    log::trace!("Executing Redis transaction...");
    redis::cmd("EXEC").query_async(&mut rconn).await
        .map_err(|_| outcome::redis_cmd_failed())?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(RouteBoardPostOutput {token}).expect("to be able to serialize RouteBoardPostOutput"))
    ))
}
