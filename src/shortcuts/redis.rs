use async_trait::async_trait;
use axum::http::StatusCode;
use crate::outcome;


#[async_trait]
pub(crate) trait RedisConnectOr504 {
    async fn get_connection_or_504(&self) -> Result<redis::aio::Connection, outcome::RequestTuple>;
}

#[async_trait]
impl RedisConnectOr504 for redis::Client {
    async fn get_connection_or_504(&self) -> Result<redis::aio::Connection, outcome::RequestTuple> {
        log::trace!("Connecting to Redis...");

        let rconn = self.get_async_connection().await
            .map_err(|_|
                (StatusCode::GATEWAY_TIMEOUT, outcome::req_error!("Could not connect to Redis"))
            )?;

        log::trace!("Connection successful!");
        Ok(rconn)
    }
}
