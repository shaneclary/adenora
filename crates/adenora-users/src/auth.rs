use adenora_common::error::AdenoraError;
use adenora_common::types::UserId;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub exp: i64,    // expiry timestamp
    pub iat: i64,    // issued at
    pub is_bot: bool,
    pub is_admin: bool,
}

pub fn create_access_token(
    user_id: UserId,
    is_bot: bool,
    is_admin: bool,
    secret: &str,
    expiry_hours: u64,
) -> Result<String, AdenoraError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + Duration::hours(expiry_hours as i64)).timestamp(),
        iat: now.timestamp(),
        is_bot,
        is_admin,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AdenoraError::Internal(format!("JWT encode error: {e}")))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AdenoraError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AdenoraError::TokenExpired,
        _ => AdenoraError::InvalidCredentials,
    })
}

pub fn hash_password(password: &str) -> Result<String, AdenoraError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AdenoraError::Internal(format!("Password hash error: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AdenoraError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash, PasswordVerifier},
    };

    let parsed = PasswordHash::new(hash)
        .map_err(|e| AdenoraError::Internal(format!("Password hash parse error: {e}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
