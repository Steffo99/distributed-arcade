//! Module defining routes for `/board/`.

use axum::{Extension, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use redis::AsyncCommands;
use serde_json::json;
use serde::Serialize;
use serde::Deserialize;


/// Possible results for `POST /board/`.
pub enum OutcomeBoardPost {
    /// Could not connect to Redis.
    RedisConnectionError,
    /// Could not check the existence of the board on Redis.
    RedisCheckExistenceError,
    /// Could not set the board ordering on Redis.
    RedisSetOrderError,
    /// Board already exists.
    AlreadyExists,
    /// Board created successfully.
    Success,
}

use OutcomeBoardPost::*;

impl IntoResponse for OutcomeBoardPost {
    fn into_response(self) -> Response {
        let (status, response) = match self {
            RedisConnectionError => (StatusCode::GATEWAY_TIMEOUT, json!("Could not connect to Redis")),
            RedisCheckExistenceError => (StatusCode::INTERNAL_SERVER_ERROR, json!("Could not check if the board already exists")),
            RedisSetOrderError => (StatusCode::INTERNAL_SERVER_ERROR, json!("Could not set the board's ordering")),
            AlreadyExists => (StatusCode::CONFLICT, json!("Board already exists")),
            Success => (StatusCode::OK, json!([]))
        };

        IntoResponse::into_response((status, Json(response)))
    }
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Order {
    /// The greater the score, the worse it is.
    Ascending,
    /// The greater the score, the better it is.
    Descending,
}


/// Expected input data for `POST /board/`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteBoardPostInput {
    name: String,
    order: Order,
}


/// Handler for `POST /board/`.
///
/// Creates a new board, storing the details on Redis.
///
/// Will refuse to overwrite an already existing board.
pub async fn route_board_post(
    Extension(rclient): Extension<redis::Client>,
    Json(input): Json<RouteBoardPostInput>,
) -> Result<OutcomeBoardPost, OutcomeBoardPost> {

    log::trace!("Connecting to Redis...");
    let mut rconn = rclient.get_async_connection().await
        .map_err(|_| RedisConnectionError)?;

    let name = &input.name;
    let order_key = format!("board:{name}:order");
    let scores_key = format!("board:{name}:scores");

    log::trace!("Checking that the board does not already exist via the order key...");
    redis::cmd("TYPE").arg(&order_key).query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(|_| RedisCheckExistenceError)?
        .eq("none").then_some(())
        .ok_or(AlreadyExists)?;

    // Possibly superfluous, but better be safe than sorry
    log::trace!("Checking that the board does not already exist via the scores key...");
    redis::cmd("TYPE").arg(&scores_key).query_async::<redis::aio::Connection, String>(&mut rconn).await
        .map_err(|_| RedisCheckExistenceError)?
        .eq("none").then_some(())
        .ok_or(AlreadyExists)?;

    log::info!("Creating board: {}", &name);

    log::trace!("Setting the board order...");
    rconn.set(&order_key, match input.order {
        Order::Ascending  => "LT",
        Order::Descending => "GT",
    }).await
        .map_err(|_| RedisSetOrderError)?;

    Ok(Success)
}
