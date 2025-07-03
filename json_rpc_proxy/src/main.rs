mod handlers;
mod services;
mod configs;
mod utils;
mod auth;
mod auth_middleware;
mod config; // ← This must be present


use crate::config::filter_config::FilterConfig;
use crate::config::reloader::{load_filter_config, start_watching_config};

use axum::{
    extract::{ConnectInfo, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use handlers::rpc::handle_rpc;
use services::rate_limiter::check_rate_limit;
use std::{net::SocketAddr, sync::{Arc, RwLock}};
use redis::Client;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use serde::{Deserialize, Serialize};
use utils::logger::init;
use auth::jwt::{Claims, encode_token};
use handlers::auth::generate_token_handler;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init();

    // Load and watch filter config
    let config_path = "config/filters.yaml";
    let shared_config = Arc::new(RwLock::new(load_filter_config()));
start_watching_config(shared_config.clone());

    // Redis
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Arc::new(Client::open(redis_url).expect("Failed to connect to Redis"));

    // Protected route with JWT and rate limiting
    let protected_routes = Router::new().route(
        "/",
        post({
            let redis_client = redis_client.clone();
            let config = shared_config.clone();
            move |ConnectInfo(addr): ConnectInfo<SocketAddr>, State(config): State<Arc<RwLock<FilterConfig>>>, body| {
                let redis_client = redis_client.clone();
                async move {
                    if let Err(res) = check_rate_limit(&redis_client, addr.ip()).await {
                        return res.into_response();
                    }
                    handle_rpc(State(config), body).await.into_response()
                }
            }
        }),
    )
    .layer(middleware::from_fn(auth_middleware::require_jwt));

    // Public route
    let app = Router::new()
        .route("/generate-token", post(generate_token_handler))
        .route("/metrics", get(|| async {
            use axum::http::{HeaderValue, header};
            use axum::response::Response;
            use axum::body::Body;

            let body = utils::metrics::metrics_handler().await;
            let mut response = Response::new(Body::from(body));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; version=0.0.4"),
            );
            response
        }))
        .merge(protected_routes)
        .with_state(shared_config) // <-- Inject config globally
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("✅ Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
