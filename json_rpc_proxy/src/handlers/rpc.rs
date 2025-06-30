use axum::{Json, extract::Extension};
use serde_json::Value;
use tracing::info;

use crate::services::proxy::forward_to_upstream;

pub async fn handle_rpc(Json(payload): Json<Value>) -> Json<Value> {
    if let Some(method) = payload.get("method").and_then(|m| m.as_str()) {
        info!("Received JSON-RPC method: {}", method);
    } else {
        info!("Received JSON-RPC call with no method field");
    }

    let response = match forward_to_upstream(payload).await {
        Ok(res) => res,
        Err(err) => {
            info!("Upstream error: {}", err);
            json_error("Upstream error")
        }
    };

    Json(response)
}

fn json_error(msg: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32000,
            "message": msg
        },
        "id": null
    })
}
