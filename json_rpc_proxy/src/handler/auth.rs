use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use crate::auth::jwt::generate_token;
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct TokenRequest {
    pub user_id: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

pub async fn generate_token_handler(
    Json(payload): Json<TokenRequest>,
) -> impl IntoResponse {
    match generate_token(&payload.user_id, &payload.role) {
        Ok(token) => {
            let response = TokenResponse { token };
            (StatusCode::OK, Json(response))
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to generate token"),
        ),
    }
}
