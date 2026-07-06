use crate::middleware::validate::{is_strong_password, is_valid_display_name, is_valid_email};
use crate::state::AppState;
use adenora_users::auth;
use adenora_users::kyc::verify_age;
use axum::{Json, extract::State, http::StatusCode};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub date_of_birth: String,
    pub country_code: String,
    pub locale: Option<String>,
    pub preferred_currency: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate input before touching the database.
    if !is_valid_email(&body.email) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "invalid email address"}))));
    }
    if let Err(msg) = is_strong_password(&body.password) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": msg}))));
    }
    if let Err(msg) = is_valid_display_name(&body.display_name) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": msg}))));
    }

    // Age check: parse the supplied date of birth and enforce the minimum age.
    let dob = NaiveDate::parse_from_str(&body.date_of_birth, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid date of birth (expected YYYY-MM-DD)"}))))?;
    if verify_age(dob, state.config.auth.min_age).is_err() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "must meet minimum age requirement"}))));
    }

    // Hash password
    let password_hash = auth::hash_password(&body.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let locale = body.locale.unwrap_or_else(|| "en".to_string());
    let currency = body.preferred_currency.unwrap_or_else(|| "EUR".to_string());

    // Insert user
    let user_id: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, password_hash, display_name, date_of_birth, country_code, locale, preferred_currency)
         VALUES ($1, $2, $3, $4::date, $5, $6, $7)
         RETURNING id"
    )
    .bind(&body.email)
    .bind(&password_hash)
    .bind(&body.display_name)
    .bind(&body.date_of_birth)
    .bind(&body.country_code)
    .bind(&locale)
    .bind(&currency)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        let msg = if e.to_string().contains("duplicate") {
            "email already registered"
        } else {
            "registration failed"
        };
        (StatusCode::BAD_REQUEST, Json(json!({"error": msg})))
    })?;

    // Create default EUR wallet
    sqlx::query("INSERT INTO wallets (user_id, currency) VALUES ($1, $2)")
        .bind(user_id.0)
        .bind(&currency)
        .execute(&state.db)
        .await
        .ok();

    // Generate tokens
    let access_token = auth::create_access_token(
        user_id.0,
        false,
        false,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_expiry_hours,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "status": "registered",
        "user_id": user_id.0,
        "access_token": access_token,
        "message": "account created — KYC verification required for trading"
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Fetch user
    let row: Option<(uuid::Uuid, String, bool, bool)> = sqlx::query_as(
        "SELECT id, password_hash, is_bot_account, is_admin FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "database error"}))))?;

    let (user_id, hash, is_bot, is_admin) = row
        .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"}))))?;

    // Verify password
    let valid = auth::verify_password(&body.password, &hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "auth error"}))))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid credentials"}))));
    }

    let access_token = auth::create_access_token(
        user_id,
        is_bot,
        is_admin,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_expiry_hours,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "status": "authenticated",
        "user_id": user_id,
        "access_token": access_token,
        "is_bot": is_bot,
        "expires_in": state.config.auth.jwt_expiry_hours * 3600
    })))
}
