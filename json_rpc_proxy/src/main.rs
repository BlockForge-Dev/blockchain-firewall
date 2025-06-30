mod handlers;
mod services;
mod router;
mod utils;

use axum::serve;
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use utils::logger::init;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init();

    let app = router::create_router();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    serve(listener, app).await.unwrap();
}
