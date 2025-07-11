use axum::{Json, http::StatusCode};
use serde_json::{Value, json};
use tracing::info;
use std::time::Instant;

use crate::config::is_blocked;
use crate::services::gas_filter::{check_gas_limit, GasFilterError};
use crate::services::proxy::forward_to_upstream;
use crate::utils::metrics::{REQUESTS, BLOCKED_REQUESTS, LATENCIES};

pub async fn handle_rpc(Json(req_body): Json<Value>) -> Result<Json<Value>, StatusCode> {
    let method = req_body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown")
        .to_string();

    info!("Received JSON-RPC method: {}", method);
    REQUESTS.with_label_values(&[&method]).inc();

    // 🔒 Check blocklist
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

    // ⛽ Gas-aware filtering for eth_sendTransaction
    if let Err(err) = check_gas_limit(&method, &req_body) {
        BLOCKED_REQUESTS.with_label_values(&["gas_filtered"]).inc();

        let (code, message) = match err {
            GasFilterError::TooLow => (-403, "Gas limit too low for this transaction"),
            GasFilterError::TooHigh => (-403, "Gas limit exceeded"),
            GasFilterError::MissingGas => (-32602, "Missing gas field in transaction"),
            GasFilterError::MissingParams => (-32602, "Missing transaction parameters"),
            GasFilterError::InvalidFormat => (-32602, "Invalid gas format"),
        };

        let error = json!({
            "jsonrpc": "2.0",
            "id": req_body.get("id").unwrap_or(&json!(null)),
            "error": {
                "code": code,
                "message": message
            }
        });
        return Ok(Json(error));
    }

    // ⏱️ Forward to upstream and measure latency
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
                    "message": "Upstream request failed"
                }
            });
            Ok(Json(error))
        }
    }
}
