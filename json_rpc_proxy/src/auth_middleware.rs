use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::auth::jwt::decode_token;

pub async fn require_jwt(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            match decode_token(token) {
                Ok(claims) => {
                    // ✅ Log user
                    tracing::info!("✅ Authenticated: {} (role: {})", claims.sub, claims.role);

                    // 🔐 Role check: Only "admin" allowed
                    if claims.role != "admin" {
                        tracing::warn!("❌ Forbidden: role '{}' is not allowed", claims.role);
                        return Err(StatusCode::FORBIDDEN);
                    }

                    return Ok(next.run(req).await);
                }
                Err(e) => {
                    tracing::warn!("❌ Invalid JWT: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    tracing::warn!("❌ Missing or malformed Authorization header");
    Err(StatusCode::UNAUTHORIZED)
}
