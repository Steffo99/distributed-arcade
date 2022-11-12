use lazy_static::lazy_static;
use std::net::SocketAddr;
use std::env;


lazy_static! {
    pub(crate) static ref REDIS_CONN: String = env::var("REDIS_CONN_STRING")
        .expect("REDIS_CONN_STRING to be set");

    pub(crate) static ref AXUM_HOST: SocketAddr = env::var("AXUM_HOST_STRING")
        .expect("AXUM_HOST_STRING to be set")
        .parse()
        .expect("AXUM_HOST_STRING to be a valid SocketAddr");

    pub(crate) static ref CREATE_TOKEN: String = env::var("CREATE_TOKEN")
        .expect("CREATE_TOKEN to be set");
}
