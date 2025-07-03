mod handlers;
mod services;
mod config;
mod utils;
mod auth;
mod auth_middleware;




use axum::{
    extract::ConnectInfo,
    middleware,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use axum::routing::get;
use handlers::rpc::handle_rpc;
use services::rate_limiter::check_rate_limit;
use std::{net::SocketAddr, sync::Arc};
use redis::Client;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use serde::{Deserialize, Serialize};
use utils::logger::init;
use auth::jwt::{Claims, encode_token};
use handlers::auth::generate_token_handler;

// #[derive(Deserialize)]
// struct TokenRequest {
//     user_id: String,
//     role: String,
// }

// #[derive(Serialize)]
// struct TokenResponse {
//     token: String,
// }

/// Public route: POST /generate-token → returns JWT
// async fn generate_token_handler(Json(payload): Json<TokenRequest>) -> impl IntoResponse {
//     use std::time::{SystemTime, UNIX_EPOCH};

//     let exp = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs()
//         + 3600; // 1 hour expiry

//     let claims = Claims {
//         sub: payload.user_id,
//         role: payload.role,
//         exp: exp as usize,
//     };

//     let token = encode_token(&claims);
//     Json(TokenResponse { token })
// }

#[tokio::main]
async fn main() {
    // Load environment variables and initialize logger
    dotenv::dotenv().ok();
    init();

    // Connect to Redis
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Arc::new(Client::open(redis_url).expect("Failed to connect to Redis"));

    // Protected route (requires JWT + rate limiting)
    let protected_routes = Router::new().route(
        "/",
        post({
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
        }),
    )
    .layer(middleware::from_fn(auth_middleware::require_jwt));


    // Public route (does not require JWT)
    let app = Router::new()
    
        .route("/generate-token", post(generate_token_handler))
   .route("/metrics", get(|| async {
    use axum::http::{HeaderValue, StatusCode, header};
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
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("✅ Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
