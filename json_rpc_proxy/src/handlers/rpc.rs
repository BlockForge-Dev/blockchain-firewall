use axum::{Json, extract::Extension};
use serde_json::{Value, json};
use tracing::info;
use crate::config::is_blocked;
use crate::services::proxy::forward_to_upstream;

pub async fn handle_rpc(Json(req_body): Json<Value>) -> Result<Json<Value>, axum::http::StatusCode> {
    // Extract method name
    let method = req_body
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    info!("Received JSON-RPC method: {}", method);

    // Check if method is blocked
    if is_blocked(method) {
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

    // Forward to upstream if not blocked
    match forward_to_upstream(req_body).await {
        Ok(resp) => Ok(Json(resp)),
        Err(_) => {
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
