mod handlers;
mod services;
mod config;
mod utils;

use axum::{
    extract::ConnectInfo,
    response::IntoResponse,
    routing::post,
    Router,
};
use handlers::rpc::handle_rpc;
use services::rate_limiter::check_rate_limit;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use redis::Client;
use utils::logger::init; // ✅ fix logger import
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init(); // ✅ call logger::init()

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Arc::new(Client::open(redis_url).expect("Failed to create Redis client"));

    // Bind a TcpListener manually as required by axum 0.7+
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    let app = Router::new()
        .route("/", post({
            let redis_client = redis_client.clone();
            move |ConnectInfo(addr): ConnectInfo<SocketAddr>, body| {
                let redis_client = redis_client.clone();
                async move {
                    if let Err(res) = check_rate_limit(&redis_client, addr.ip()).await {
                        return res.into_response();
                    }
                    handle_rpc(body).await.into_response()
                }
            }
        }))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    tracing::info!("✅ Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
