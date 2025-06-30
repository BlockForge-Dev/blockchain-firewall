use reqwest::Client;
use serde_json::Value;
use std::env;
use anyhow::{Result, anyhow};
use tracing::{error, info};

pub async fn forward_to_upstream(payload: Value) -> Result<Value> {
    let url = env::var("UPSTREAM_URL")
        .map_err(|_| anyhow!("UPSTREAM_URL not set"))?;

    info!("Forwarding to upstream: {}", url);

    let client = Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            info!("Upstream status: {}", status);
            info!("Upstream raw body: {}", body);

            match serde_json::from_str::<Value>(&body) {
                Ok(json) => Ok(json),
                Err(e) => {
                    error!("JSON parse error: {} | raw body: {}", e, body);
                    Err(anyhow!("Invalid JSON from upstream: {}", e))
                }
            }
        },
        Err(e) => {
            error!("HTTP request failed: {}", e);
            Err(anyhow!("Failed to contact upstream: {}", e))
        }
    }
}
