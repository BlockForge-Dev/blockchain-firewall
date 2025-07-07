mod handlers;
mod services;
mod configs;
mod utils;
mod auth;
mod auth_middleware;
mod config;
mod wasm;
mod state;

use crate::config::reloader::{load_filter_config, start_watching_config};
use crate::state::AppState;
use crate::wasm::plugin_engine::WasmPlugin;

use axum::{
    extract::{ConnectInfo, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use handlers::rpc::handle_rpc;
use services::rate_limiter::check_rate_limit;

use std::{
    net::SocketAddr,
    sync::{Arc, RwLock, Mutex},
};

use redis::Client;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use utils::logger::init;
use handlers::auth::generate_token_handler;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init();

    // ✅ Load filter config & start live reloading
    let config_path = "config/filters.yaml";
    let shared_config = Arc::new(RwLock::new(load_filter_config()));
    start_watching_config(shared_config.clone());

    // ✅ Load WASM plugin and wrap in Arc<Mutex<_>>
    let plugin = Arc::new(Mutex::new(
        WasmPlugin::load("firewall_plugin.wasm").expect("❌ Failed to load plugin"),
    ));

    // ✅ Create shared AppState
    let state = AppState {
        config: shared_config.clone(),
        plugin,
    };

    // ✅ Setup Redis
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Arc::new(Client::open(redis_url).expect("Failed to connect to Redis"));

    // ✅ Protected RPC route with rate limiting and JWT auth
    let protected_routes = Router::new().route(
        "/",
        post({
            let redis_client = redis_client.clone();
            move |ConnectInfo(addr): ConnectInfo<SocketAddr>, State(state): State<AppState>, body| {
                let redis_client = redis_client.clone();
                async move {
                    if let Err(res) = check_rate_limit(&redis_client, addr.ip()).await {
                        return res.into_response();
                    }
                    handle_rpc(State(state), body).await.into_response()
                }
            }
        }),
    )
    .layer(middleware::from_fn(auth_middleware::require_jwt));

    // ✅ Main app routes
    let app = Router::new()
        .route("/generate-token", post(generate_token_handler))
        .route("/metrics", get(|| async {
            use axum::{
                body::Body,
                http::{header, HeaderValue},
                response::Response,
            };

            let body = utils::metrics::metrics_handler().await;
            let mut response = Response::new(Body::from(body));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; version=0.0.4"),
            );
            response
        }))
        .merge(protected_routes)
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    // ✅ Launch Axum server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("✅ Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
