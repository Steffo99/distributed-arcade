pub(crate) mod config;
pub(crate) mod outcome;
pub mod utils;
mod routes;
mod shortcuts;


use axum::routing::{get, post, put};


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
        .route("/", post(routes::home::route_home_post))
        .route("/board/", get(routes::board::route_board_get))
        .route("/board/", post(routes::board::route_board_post))
        .route("/score/", get(routes::score::route_score_get))
        .route("/score/", put(routes::score::route_score_put))
        .layer(axum::Extension(rclient))
        .layer(tower_http::cors::CorsLayer::new()
            .allow_origin("*".parse::<axum::http::HeaderValue>().expect("* to be a valid origin"))
        );

    log::info!("Starting Axum server...");

    axum::Server::bind(&config::AXUM_HOST).serve(webapp.into_make_service()).await
        .expect("to be able to run the Axum server");
}
