// services/multichain/adapter.rs
use axum::http::StatusCode;
use serde_json::Value;

#[async_trait::async_trait]
pub trait ChainAdapter: Send + Sync {
    async fn forward(&self, payload: Value) -> Result<Value, StatusCode>;
}
