use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::auth::jwt::decode_token;

pub async fn require_jwt(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req.headers().get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    if let Some(auth_header) = auth_header {
        // Check for Bearer token prefix
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Decode and validate the token
            match decode_token(token) {
                Ok(claims) => {
                    tracing::info!("✅ Authenticated user: {} (role: {})", claims.sub, claims.role);
                    return Ok(next.run(req).await);
                }
                Err(e) => {
                    tracing::warn!("❌ Invalid JWT: {}", e);
                    return Ok((StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response());
                }
            }
        }
    }

    tracing::warn!("❌ Missing or malformed Authorization header");
    Ok((StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header").into_response())
}
