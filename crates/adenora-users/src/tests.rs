use crate::auth;

#[test]
fn test_password_hash_and_verify() {
    let password = "TestPassword123";
    let hash = auth::hash_password(password).unwrap();
    assert_ne!(hash, password);
    assert!(auth::verify_password(password, &hash).unwrap());
}

#[test]
fn test_wrong_password_fails() {
    let hash = auth::hash_password("CorrectPassword1").unwrap();
    assert!(!auth::verify_password("WrongPassword1", &hash).unwrap());
}

#[test]
fn test_jwt_create_and_validate() {
    let user_id = uuid::Uuid::new_v4();
    let secret = "test-secret-key-for-jwt";

    let token = auth::create_access_token(user_id, false, false, secret, 24).unwrap();
    assert!(!token.is_empty());

    let claims = auth::validate_token(&token, secret).unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert!(!claims.is_bot);
    assert!(!claims.is_admin);
}

#[test]
fn test_jwt_bot_flag() {
    let user_id = uuid::Uuid::new_v4();
    let secret = "test-secret";

    let token = auth::create_access_token(user_id, true, false, secret, 1).unwrap();
    let claims = auth::validate_token(&token, secret).unwrap();
    assert!(claims.is_bot);
    assert!(!claims.is_admin);
}

#[test]
fn test_jwt_admin_flag() {
    let user_id = uuid::Uuid::new_v4();
    let secret = "test-secret";

    let token = auth::create_access_token(user_id, false, true, secret, 1).unwrap();
    let claims = auth::validate_token(&token, secret).unwrap();
    assert!(!claims.is_bot);
    assert!(claims.is_admin);
}

#[test]
fn test_jwt_wrong_secret_fails() {
    let user_id = uuid::Uuid::new_v4();
    let token = auth::create_access_token(user_id, false, false, "secret1", 1).unwrap();
    let result = auth::validate_token(&token, "wrong-secret");
    assert!(result.is_err());
}

#[test]
fn test_jwt_expired_token() {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use crate::auth::Claims;

    let secret = "test-secret";
    let claims = Claims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: 1000, // Unix timestamp from 1970 — definitely expired
        iat: 999,
        is_bot: false,
        is_admin: false,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();
    let result = auth::validate_token(&token, secret);
    assert!(result.is_err());
}
