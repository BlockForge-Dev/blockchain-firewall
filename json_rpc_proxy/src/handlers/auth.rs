use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use crate::auth::jwt::{encode_token, Claims};

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
    use std::time::{SystemTime, UNIX_EPOCH};

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600;

    let claims = Claims {
        sub: payload.user_id.clone(),
        role: payload.role.clone(),
        exp: exp as usize,
    };

    let token = encode_token(&claims);
    let response = TokenResponse { token };

    (StatusCode::OK, Json(response))
}

