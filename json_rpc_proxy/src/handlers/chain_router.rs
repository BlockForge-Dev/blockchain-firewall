// handlers/chain_router.rs
use axum::{extract::Path, Json};
use serde_json::Value;
use crate::services::multichain::{ChainId, get_adapter};

pub async fn chain_router(
    Path(chain): Path<String>,
    Json(req): Json<Value>,
) -> Result<Json<Value>, axum::http::StatusCode> {
    let chain_id = ChainId::from_path(&chain).ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    let adapter = get_adapter(&chain_id);

    let result = adapter.forward(req).await?;
    Ok(Json(result))
}
