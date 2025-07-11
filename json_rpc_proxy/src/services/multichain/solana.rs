// services/multichain/solana.rs
use axum::http::StatusCode;
use serde_json::Value;
use crate::services::multichain::adapter::ChainAdapter;

pub struct SolanaAdapter;

#[async_trait::async_trait]
impl ChainAdapter for SolanaAdapter {
    async fn forward(&self, payload: Value) -> Result<Value, StatusCode> {
        let client = reqwest::Client::new();
        let upstream_url = "https://api.mainnet-beta.solana.com"; // Replace with real endpoint

        match client.post(upstream_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => Ok(json),
                Err(err) => {
                    tracing::error!("Solana JSON parse error: {:?}", err);
                    Err(StatusCode::BAD_GATEWAY)
                }
            },
            Err(err) => {
                tracing::error!("Solana upstream error: {:?}", err);
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }
}
