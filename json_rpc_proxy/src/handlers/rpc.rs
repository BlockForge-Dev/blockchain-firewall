use axum::Json;
use serde_json::{Value, json};
use tracing::info;
use crate::config::is_blocked;
use crate::services::proxy::forward_to_upstream;
use crate::utils::metrics::{REQUESTS, BLOCKED_REQUESTS, LATENCIES};

use std::time::Instant;

pub async fn handle_rpc(Json(req_body): Json<Value>) -> Result<Json<Value>, axum::http::StatusCode> {
    // Extract method as String
    let method = req_body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string(); // ✅ convert to owned String early

    info!("Received JSON-RPC method: {}", method);
    REQUESTS.with_label_values(&[&method]).inc();

    // Check blocklist
    if is_blocked(&method) {
        BLOCKED_REQUESTS.with_label_values(&["method_blocked"]).inc();
        

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

    // Track latency
    let start = Instant::now();

    match forward_to_upstream(req_body).await {
        Ok(resp) => {
            let duration = start.elapsed().as_secs_f64();
            LATENCIES.with_label_values(&[&method]).observe(duration);
            Ok(Json(resp))
        }
        Err(_) => {
            BLOCKED_REQUESTS.with_label_values(&["upstream_error"]).inc();

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
