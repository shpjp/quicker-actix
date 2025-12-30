use actix_web::{dev::ServiceRequest, Error};
use actix_web::error::ErrorUnauthorized;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String) -> Self {
        let now = Utc::now();
        let exp = now + Duration::days(7); // Token valid for 7 days

        Claims {
            sub: user_id.to_string(),
            email,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn create_jwt(user_id: Uuid, email: String, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, email);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn extract_user_id_from_request(req: &ServiceRequest, jwt_secret: &str) -> Result<Uuid, Error> {
    // Get Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ErrorUnauthorized("Missing authorization header"))?;

    // Extract token from "Bearer <token>"
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ErrorUnauthorized("Invalid authorization format"))?;

    // Decode JWT
    let claims = decode_jwt(token, jwt_secret)
        .map_err(|_| ErrorUnauthorized("Invalid or expired token"))?;

    // Parse user_id from claims
    Uuid::parse_str(&claims.sub)
        .map_err(|_| ErrorUnauthorized("Invalid user ID in token"))
}

// Middleware helper to extract user_id from Authorization header
pub fn get_user_id_from_token(auth_header: Option<&str>, jwt_secret: &str) -> Result<Uuid, String> {
    let auth_header = auth_header.ok_or("Missing authorization header")?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or("Invalid authorization format")?;

    let claims = decode_jwt(token, jwt_secret)
        .map_err(|_| "Invalid or expired token")?;

    Uuid::parse_str(&claims.sub)
        .map_err(|_| "Invalid user ID in token".to_string())
}
