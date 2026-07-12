// Interface scaffolding: the Veriff KYC provider interface (request/webhook DTOs
// and helpers) is retained for a planned verification integration; several fields
// are deserialized from the provider but not yet consumed.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// KYC verification service.
/// In production, this integrates with Veriff (Estonian company, ~2 EUR/verification).
/// For now, provides the interface and webhook handler.

#[derive(Debug, Serialize)]
pub struct KycSessionRequest {
    pub user_id: Uuid,
    pub callback_url: String,
    pub locale: String,
    pub document_country: String,
}

#[derive(Debug, Deserialize)]
pub struct KycWebhookPayload {
    pub session_id: String,
    pub status: String, // "approved", "declined", "resubmission_requested"
    pub user_id: String,
    pub verification: Option<KycVerificationData>,
}

#[derive(Debug, Deserialize)]
pub struct KycVerificationData {
    pub person: Option<KycPerson>,
    pub document: Option<KycDocument>,
}

#[derive(Debug, Deserialize)]
pub struct KycPerson {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub date_of_birth: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KycDocument {
    pub document_type: Option<String>,
    pub country: Option<String>,
    pub number: Option<String>,
}

/// Process a KYC webhook callback from Veriff.
pub async fn process_webhook(
    db: &sqlx::PgPool,
    payload: &KycWebhookPayload,
) -> anyhow::Result<()> {
    let user_id = Uuid::parse_str(&payload.user_id)?;

    let new_status = match payload.status.as_str() {
        "approved" => "verified",
        "declined" => "rejected",
        "resubmission_requested" => "pending",
        _ => "none",
    };

    sqlx::query("UPDATE users SET kyc_status = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status)
        .bind(user_id)
        .execute(db)
        .await?;

    // If approved and we have DOB, update it
    if new_status == "verified" {
        if let Some(ref verification) = payload.verification {
            if let Some(ref person) = verification.person {
                if let Some(ref dob) = person.date_of_birth {
                    sqlx::query("UPDATE users SET date_of_birth = $1::date WHERE id = $2")
                        .bind(dob)
                        .bind(user_id)
                        .execute(db)
                        .await
                        .ok();
                }
            }
        }
    }

    tracing::info!(user_id = %user_id, status = new_status, "KYC status updated");
    Ok(())
}

/// Check if a user has completed KYC (required for trading and lottery).
pub async fn is_kyc_verified(db: &sqlx::PgPool, user_id: Uuid) -> bool {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT kyc_status FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .ok()
    .flatten();

    matches!(row.as_ref().map(|(s,)| s.as_str()), Some("verified"))
}
