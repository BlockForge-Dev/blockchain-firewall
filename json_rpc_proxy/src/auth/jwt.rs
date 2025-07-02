use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn encode_token(claims: &Claims) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("Failed to encode token")
}

pub fn decode_token(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}


#[cfg(test)]
#[test]
fn generate_token() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600;

    let claims = Claims {
        sub: "user@example.com".to_string(),
        role: "admin".to_string(),
        exp: exp as usize,
    };

    let token = encode_token(&claims);
    println!("JWT: {}", token);
}

