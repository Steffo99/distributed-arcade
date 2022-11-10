use axum::routing::{get, put};
use axum::Json;


#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(get_home))
        .route("/leaderboard", get(get_leaderboard))
        .route("/score", get(get_score))
        .route("/score", put(update_score));

    axum::Server::bind(&"0.0.0.0:30000".parse().unwrap())
        .serve(app.into_make_service()).await.unwrap();
}


async fn get_home() -> String {
    String::from("Hello world!")
}


async fn get_leaderboard() {
    todo!()
}


async fn get_score() {
    todo!()
}


async fn update_score() {
    todo!()
}
