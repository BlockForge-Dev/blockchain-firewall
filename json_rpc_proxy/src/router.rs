use axum::{Router, routing::post};
use crate::handlers::rpc::handle_rpc;

pub fn create_router() -> Router {
    Router::new().route("/", post(handle_rpc))
}
