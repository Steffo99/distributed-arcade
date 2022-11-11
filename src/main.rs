mod config;
mod routes;
mod outcome;

use axum::routing::{get, post};


#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::debug!("Logging initialized!");

    log::debug!("Opening Redis client...");

    let rclient = redis::Client::open(&**config::REDIS_CONN)
        .expect("to be able to connect to Redis");

    log::debug!("Configuring Axum router...");

    let webapp = axum::Router::new()
        .route("/", get(routes::home::route_home_get))
        .route("/board/", post(routes::board::route_board_post))
        .layer(axum::Extension(rclient));

    log::info!("Starting Axum server...");

    axum::Server::bind(&config::AXUM_HOST).serve(webapp.into_make_service()).await
        .expect("to be able to run the Axum server");
}
