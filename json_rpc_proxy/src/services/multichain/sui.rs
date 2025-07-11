// services/multichain/sui.rs
use axum::http::StatusCode;
use serde_json::Value;
use crate::services::multichain::adapter::ChainAdapter;

pub struct SuiAdapter;

#[async_trait::async_trait]
impl ChainAdapter for SuiAdapter {
    async fn forward(&self, payload: Value) -> Result<Value, StatusCode> {
        let client = reqwest::Client::new();
        let upstream_url = "https://fullnode.mainnet.sui.io"; // Replace with actual endpoint

        match client.post(upstream_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => Ok(json),
                Err(err) => {
                    tracing::error!("Sui JSON parse error: {:?}", err);
                    Err(StatusCode::BAD_GATEWAY)
                }
            },
            Err(err) => {
                tracing::error!("Sui upstream error: {:?}", err);
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }
}
