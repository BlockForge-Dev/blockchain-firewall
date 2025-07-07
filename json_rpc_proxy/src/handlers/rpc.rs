use axum::{extract::State, Json};
use serde_json::{json, Value};
use tracing::{error, info};

use crate::state::AppState;
use crate::services::proxy::forward_to_upstream;
use crate::utils::metrics::{BLOCKED_REQUESTS, LATENCIES, REQUESTS};

use std::time::Instant;

pub async fn handle_rpc(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    let method = req_body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    info!("Received JSON-RPC method: {}", method);
    REQUESTS.with_label_values(&[&method]).inc();

    // Check config + plugin
   let is_blocked = {
    let cfg = state.config.read().unwrap();

    let plugin_result = state
        .plugin
        .lock()
        .map(|mut plugin| plugin.should_allow(&method))
        .unwrap_or(false);

    cfg.blocked_methods.contains(&method) || !plugin_result
};

    // If blocked, return error response
    if is_blocked {
        BLOCKED_REQUESTS
            .with_label_values(&["method_blocked"])
            .inc();
        let error = json!({
            "jsonrpc": "2.0",
            "id": req_body.get("id").unwrap_or(&json!(null)),
            "error": {
                "code": -403,
                "message": format!("Method '{}' is blocked by proxy", method)
            }
        });
        return Ok(Json(error));
    }

    // Forward to upstream
    let start = Instant::now();
    match forward_to_upstream(req_body).await {
        Ok(resp) => {
            let duration = start.elapsed().as_secs_f64();
            LATENCIES.with_label_values(&[&method]).observe(duration);
            Ok(Json(resp))
        }
        Err(err) => {
            error!("Failed to forward to upstream: {}", err);
            BLOCKED_REQUESTS
                .with_label_values(&["upstream_error"])
                .inc();
            let error = json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {
                    "code": -32046,
                    "message": "Cannot fulfill request"
                }
            });
            Ok(Json(error))
        }
    }
}
