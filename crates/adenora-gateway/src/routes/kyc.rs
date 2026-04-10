use axum::{
    Json,
    extract::State,
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};

use crate::auth_extractor::AuthUser;
use crate::services::kyc::{self, KycWebhookPayload};
use crate::state::AppState;

/// POST /api/v1/kyc/start — initiate a KYC session for the authenticated user.
pub async fn start_kyc(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<StartKycRequest>,
) -> impl IntoResponse {
    let domain = &state.config.server.cors_origins.first().cloned()
        .unwrap_or_else(|| "https://adenora.app".to_string());

    let callback_url = format!("{}/api/v1/kyc/webhook", domain);

    // In production, call Veriff API here with VERIFF_API_KEY.
    // For now, return a mock session URL and set status to "pending".
    sqlx::query("UPDATE users SET kyc_status = 'pending', updated_at = NOW() WHERE id = $1")
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .ok();

    tracing::info!(
        user_id = %auth.user_id,
        locale = %req.locale,
        callback_url = %callback_url,
        "KYC session initiated"
    );

    Json(StartKycResponse {
        session_url: format!("https://magic.veriff.me/v/sample?uid={}", auth.user_id),
        status: "pending".to_string(),
    })
}

/// POST /api/v1/kyc/webhook — Veriff webhook callback with HMAC-SHA256 signature verification.
pub async fn kyc_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let secret = &state.config.kyc.webhook_secret;

    // Verify HMAC signature from X-HMAC-Signature header
    if let Some(sig_header) = headers.get("x-hmac-signature").and_then(|v| v.to_str().ok()) {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC key length is always valid");
        mac.update(&body);

        let expected = hex::encode(mac.finalize().into_bytes());
        if sig_header != expected {
            tracing::warn!("KYC webhook HMAC signature mismatch");
            return StatusCode::UNAUTHORIZED;
        }
    } else if secret != "CHANGE_ME_IN_PRODUCTION" {
        // In production with a real secret, reject unsigned requests
        tracing::warn!("KYC webhook missing HMAC signature header");
        return StatusCode::UNAUTHORIZED;
    }

    // Parse the body now that it's verified
    let payload: KycWebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "KYC webhook body parse failed");
            return StatusCode::BAD_REQUEST;
        }
    };

    match kyc::process_webhook(&state.db, &payload).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            tracing::error!(error = %e, "KYC webhook processing failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// GET /api/v1/kyc/status — check current KYC status for authenticated user.
pub async fn kyc_status(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT kyc_status FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let status = row.map(|(s,)| s).unwrap_or_else(|| "none".to_string());
    Json(KycStatusResponse { status })
}

#[derive(Deserialize)]
pub struct StartKycRequest {
    pub locale: String,
    pub document_country: String,
}

#[derive(Serialize)]
pub struct StartKycResponse {
    pub session_url: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct KycStatusResponse {
    pub status: String,
}
