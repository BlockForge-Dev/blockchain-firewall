// services/multichain/ethereum.rs
use axum::http::StatusCode;
use serde_json::Value;
use crate::services::multichain::adapter::ChainAdapter;
use crate::services::proxy::forward_to_upstream;

pub struct EthereumAdapter;

#[async_trait::async_trait]
impl ChainAdapter for EthereumAdapter {
    async fn forward(&self, payload: Value) -> Result<Value, StatusCode> {
        forward_to_upstream(payload)
            .await
            .map_err(|e| {
                tracing::error!("Ethereum adapter upstream error: {:?}", e);
                StatusCode::BAD_GATEWAY
            })
    }
}
