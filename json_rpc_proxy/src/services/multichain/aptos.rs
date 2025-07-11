// services/multichain/aptos.rs
use axum::http::StatusCode;
use serde_json::Value;
use crate::services::multichain::adapter::ChainAdapter;

pub struct AptosAdapter;

#[async_trait::async_trait]
impl ChainAdapter for AptosAdapter {
    async fn forward(&self, payload: Value) -> Result<Value, StatusCode> {
        let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");

        match method {
            "get_account" => {
                let address = payload.get("params")
                    .and_then(|p| p.get(0))
                    .and_then(|v| v.as_str())
                    .ok_or(StatusCode::BAD_REQUEST)?;

                let url = format!("https://fullnode.mainnet.aptoslabs.com/v1/accounts/{}", address);
                let res = reqwest::get(&url).await.map_err(|e| {
                    tracing::error!("Aptos upstream fetch error: {:?}", e);
                    StatusCode::BAD_GATEWAY
                })?;
                let json = res.json::<Value>().await.map_err(|e| {
                    tracing::error!("Aptos JSON parse error: {:?}", e);
                    StatusCode::BAD_GATEWAY
                })?;
                Ok(json)
            }

            _ => {
                tracing::warn!("❌ Unsupported Aptos method: {}", method);
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }
}
