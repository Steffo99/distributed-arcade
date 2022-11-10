mod config;

use axum::routing::{get, put};
use axum::{Json, Extension};
use redis::AsyncCommands;


#[tokio::main]
async fn main() {
    let rclient = redis::Client::open(&**config::REDIS_CONN)
        .expect("to be able to connect to Redis");

    let webapp = axum::Router::new()
        .route("/", get(get_home))
        .route("/leaderboard", get(get_leaderboard))
        .route("/score", get(get_score))
        .route("/score", put(update_score))
        .layer(Extension(rclient));

    axum::Server::bind(&config::AXUM_HOST).serve(webapp.into_make_service()).await
        .expect("to be able to run the Axum server");
}


async fn get_home(
    Extension(rclient): Extension<redis::Client>
) -> String {

    let mut rconn = rclient.get_async_connection().await
        .expect("to be able to create a Redis connection");

    rconn.set::<&str, &str, String>("hello", "world").await
        .expect("to be able to set things in redis");

    let world: String = rconn.get("hello").await
        .expect("to be able to get things from redis");

    format!("Hello {world}!")
}


async fn get_leaderboard(
    Extension(rclient): Extension<redis::Client>
) {
    todo!()
}


async fn get_score(
    Extension(rclient): Extension<redis::Client>
) {
    todo!()
}


async fn update_score(
    Extension(rclient): Extension<redis::Client>
) {
    todo!()
}
