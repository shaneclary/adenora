use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth_extractor::AuthUser;
use crate::state::AppState;

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
fn err(s: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (s, Json(json!({"error": msg})))
}

/// Core columns for a campaign list entry.
type CampaignListRow = (
    Uuid, String, String, String, String,
    Decimal, String, Decimal, i32, String, bool,
);

/// Extra columns fetched per campaign in the list view.
type CampaignListExtrasRow = (
    Option<String>, Option<String>, Option<String>,
    Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>,
);

/// Core columns for a single campaign detail (includes description).
type CampaignDetailRow = (
    Uuid, String, String, String, String, String,
    Decimal, String, Decimal, i32, String, bool,
);

/// Extra columns for a single campaign detail.
type CampaignDetailExtrasRow = (
    Option<String>, Option<String>, Option<String>, Option<String>,
    Option<String>, Option<String>,
    Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>,
);

/// Row for a campaign update entry.
type CampaignUpdateRow = (
    Uuid, String, String, Option<String>, Option<i32>, String,
    chrono::DateTime<chrono::Utc>,
);

/// Row for a campaign donor-wall entry.
type CampaignDonorRow = (
    String, Decimal, Option<String>, String, chrono::DateTime<chrono::Utc>,
);

/// GET /api/v1/campaigns — list active cause campaigns
pub async fn list_campaigns(State(state): State<AppState>) -> ApiResult {
    // Split into two queries to stay within sqlx 16-column tuple limit
    let rows: Vec<CampaignListRow> = sqlx::query_as(
        "SELECT id, slug, title, tagline, category,
                goal_amount, currency, amount_raised, donor_count,
                status, is_featured
         FROM cause_campaigns
         WHERE status IN ('active','funded')
         ORDER BY is_featured DESC, amount_raised DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let mut campaigns: Vec<Value> = Vec::with_capacity(rows.len());
    for (id, slug, title, tagline, category, goal, currency, raised, donors, status, featured) in rows {
        // Fetch the extra fields separately
        let extras: Option<CampaignListExtrasRow> = sqlx::query_as(
            "SELECT hero_image_url, impact_metric, impact_value,
                    location, partner_org, ends_at
             FROM cause_campaigns WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

        let (hero, metric, metric_val, location, partner, ends_at) =
            extras.unwrap_or_default();

        let pct = if goal > Decimal::ZERO {
            ((raised / goal) * Decimal::from(100)).min(Decimal::from(100))
        } else { Decimal::ZERO };

        campaigns.push(json!({
            "id": id, "slug": slug, "title": title, "tagline": tagline,
            "category": category, "goal_amount": goal, "currency": currency,
            "amount_raised": raised, "donor_count": donors,
            "progress_pct": pct, "hero_image_url": hero,
            "impact_metric": metric, "impact_value": metric_val,
            "location": location, "partner_org": partner,
            "status": status, "is_featured": featured, "ends_at": ends_at,
        }));
    }

    Ok(Json(json!({ "campaigns": campaigns, "total": campaigns.len() })))
}

/// GET /api/v1/campaigns/:slug — single campaign with updates + donor wall
pub async fn get_campaign(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult {
    // Core fields (≤16 cols)
    let core: Option<CampaignDetailRow> = sqlx::query_as(
        "SELECT id, slug, title, tagline, category, description,
                goal_amount, currency, amount_raised, donor_count,
                status, is_featured
         FROM cause_campaigns WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (id, slug, title, tagline, category, description, goal, currency, raised, donors, status, featured) =
        core.ok_or_else(|| err(StatusCode::NOT_FOUND, "campaign not found"))?;

    // Extra fields
    let extras: Option<CampaignDetailExtrasRow> = sqlx::query_as(
        "SELECT hero_image_url, impact_metric, impact_value, location,
                partner_org, partner_url, ends_at, funded_at
         FROM cause_campaigns WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (hero, metric, metric_val, location, partner, partner_url, ends_at, funded_at) =
        extras.unwrap_or_default();

    let pct = if goal > Decimal::ZERO {
        ((raised / goal) * Decimal::from(100)).min(Decimal::from(100))
    } else { Decimal::ZERO };

    // Updates
    let updates: Vec<CampaignUpdateRow> =
        sqlx::query_as(
            "SELECT id, title, body, image_url, milestone_pct, author, published_at
             FROM campaign_updates WHERE campaign_id = $1
             ORDER BY published_at DESC LIMIT 20"
        )
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let updates_json: Vec<Value> = updates.into_iter().map(|(uid, t, b, img, ms, auth, pub_at)| json!({
        "id": uid, "title": t, "body": b,
        "image_url": img, "milestone_pct": ms,
        "author": auth, "published_at": pub_at
    })).collect();

    // Donor wall (top 20, visible only)
    let wall: Vec<(String, Decimal, Option<String>, String)> =
        sqlx::query_as(
            "SELECT display_name, total_given, message, badge
             FROM donor_wall WHERE campaign_id = $1 AND is_visible = true
             ORDER BY total_given DESC LIMIT 20"
        )
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let wall_json: Vec<Value> = wall.into_iter().map(|(name, given, msg, badge)| json!({
        "display_name": name,
        "total_given": given,
        "message": msg,
        "badge": badge,
    })).collect();

    Ok(Json(json!({
        "id": id,
        "slug": slug,
        "title": title,
        "tagline": tagline,
        "category": category,
        "description": description,
        "goal_amount": goal,
        "currency": currency,
        "amount_raised": raised,
        "donor_count": donors,
        "progress_pct": pct,
        "hero_image_url": hero,
        "impact_metric": metric,
        "impact_value": metric_val,
        "location": location,
        "partner_org": partner,
        "partner_url": partner_url,
        "status": status,
        "is_featured": featured,
        "ends_at": ends_at,
        "funded_at": funded_at,
        "updates": updates_json,
        "donor_wall": wall_json,
    })))
}

#[derive(Deserialize)]
pub struct DonateRequest {
    pub campaign_id: Uuid,
    pub amount: Decimal,
    pub currency: Option<String>,
    pub message: Option<String>,
    pub is_anonymous: Option<bool>,
    pub display_name: Option<String>,
}

/// POST /api/v1/campaigns/donate — direct donation from wallet
pub async fn donate(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<DonateRequest>,
) -> ApiResult {
    if req.amount <= Decimal::ZERO {
        return Err(err(StatusCode::BAD_REQUEST, "amount must be positive"));
    }

    // KYC check — must be verified to donate
    let kyc: Option<(String,)> = sqlx::query_as(
        "SELECT kyc_status FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    match kyc.as_ref().map(|(s,)| s.as_str()) {
        Some("verified") | Some("approved") => {}
        Some(status) => return Err(err(StatusCode::FORBIDDEN, &format!("KYC verification required (current: {})", status))),
        None => return Err(err(StatusCode::UNAUTHORIZED, "user not found")),
    }

    let currency = req.currency.as_deref().unwrap_or("EUR");
    let is_anon = req.is_anonymous.unwrap_or(false);

    // Check campaign exists and is active
    let campaign: Option<(Uuid, String, Decimal, String)> = sqlx::query_as(
        "SELECT id, status, goal_amount, currency FROM cause_campaigns WHERE id = $1"
    )
    .bind(req.campaign_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (camp_id, status, ..) = campaign
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "campaign not found"))?;

    if status != "active" {
        return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "campaign is not accepting donations"));
    }

    // Debit wallet — donations come from available balance, no reserve needed
    let updated: Option<(Decimal,)> = sqlx::query_as(
        "UPDATE wallets SET available = available - $1, updated_at = NOW()
         WHERE user_id = $2 AND currency = $3 AND available >= $1
         RETURNING available"
    )
    .bind(req.amount)
    .bind(auth.user_id)
    .bind(currency)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if updated.is_none() {
        return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient balance"));
    }

    // Get display name
    let donor_name: Option<(String,)> = sqlx::query_as(
        "SELECT display_name FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let display = if is_anon {
        "Anonymous".to_string()
    } else {
        req.display_name
            .or(donor_name.map(|(n,)| n))
            .unwrap_or_else(|| "Anonymous".to_string())
    };

    // Resolve charity project for this campaign — required for ledger integrity
    let project_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT cp.id FROM cause_campaigns cc
         JOIN charity_projects cp
           ON cp.name ILIKE '%' || cc.partner_org || '%'
         WHERE cc.id = $1
         LIMIT 1"
    )
    .bind(camp_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Fall back to the first active project if no partner match — still must exist
    let resolved_project: (Uuid,) = match project_id {
        Some(p) => p,
        None => sqlx::query_as(
            "SELECT id FROM charity_projects WHERE is_active = true ORDER BY created_at LIMIT 1"
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
        .ok_or_else(|| err(StatusCode::INTERNAL_SERVER_ERROR, "no charity project available to receive funds"))?,
    };

    // Record charity ledger entry — mandatory, returns ID for donation link
    let ledger_entry: (Uuid,) = sqlx::query_as(
        "INSERT INTO charity_ledger (project_id, source, amount, currency, description, reference_id)
         VALUES ($1, 'direct_donation', $2, $3, $4, $5::text)
         RETURNING id"
    )
    .bind(resolved_project.0)
    .bind(req.amount)
    .bind(currency)
    .bind(format!("direct donation to campaign {}", camp_id))
    .bind(camp_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Keep project total in sync
    sqlx::query(
        "UPDATE charity_projects SET total_received = total_received + $1 WHERE id = $2"
    )
    .bind(req.amount)
    .bind(resolved_project.0)
    .execute(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Record donation — linked to ledger entry
    let donation_id: (Uuid,) = sqlx::query_as(
        "INSERT INTO direct_donations
            (campaign_id, user_id, amount, currency, message, is_anonymous, display_name, status, ledger_entry_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed', $8)
         RETURNING id"
    )
    .bind(camp_id)
    .bind(auth.user_id)
    .bind(req.amount)
    .bind(currency)
    .bind(&req.message)
    .bind(is_anon)
    .bind(&display)
    .bind(ledger_entry.0)
    .fetch_one(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Upsert donor wall
    if !is_anon {
        sqlx::query(
            "INSERT INTO donor_wall (campaign_id, user_id, display_name, total_given, currency, message, badge)
             VALUES ($1, $2, $3, $4, $5, $6,
                CASE WHEN $4 >= 10000 THEN 'Champion'
                     WHEN $4 >= 1000  THEN 'Founder'
                     WHEN $4 >= 100   THEN 'Supporter'
                     ELSE '' END)
             ON CONFLICT (campaign_id, user_id) DO UPDATE
             SET total_given = donor_wall.total_given + EXCLUDED.total_given,
                 message = COALESCE(EXCLUDED.message, donor_wall.message),
                 last_donated_at = NOW(),
                 badge = CASE WHEN donor_wall.total_given + EXCLUDED.total_given >= 10000 THEN 'Champion'
                              WHEN donor_wall.total_given + EXCLUDED.total_given >= 1000  THEN 'Founder'
                              WHEN donor_wall.total_given + EXCLUDED.total_given >= 100   THEN 'Supporter'
                              ELSE '' END"
        )
        .bind(camp_id)
        .bind(auth.user_id)
        .bind(&display)
        .bind(req.amount)
        .bind(currency)
        .bind(&req.message)
        .execute(&state.db)
        .await
        .ok();
    }

    tracing::info!(
        user_id = %auth.user_id,
        campaign_id = %camp_id,
        amount = %req.amount,
        "donation recorded"
    );

    Ok(Json(json!({
        "donation_id": donation_id.0,
        "campaign_id": camp_id,
        "amount": req.amount,
        "currency": currency,
        "message": "Thank you. Your contribution flows.",
    })))
}

/// GET /api/v1/campaigns/:slug/donors — public donor wall
pub async fn get_donors(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult {
    let campaign_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM cause_campaigns WHERE slug = $1"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let id = campaign_id
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "campaign not found"))?.0;

    let wall: Vec<CampaignDonorRow> =
        sqlx::query_as(
            "SELECT display_name, total_given, message, badge, last_donated_at
             FROM donor_wall WHERE campaign_id = $1 AND is_visible = true
             ORDER BY total_given DESC LIMIT 50"
        )
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let donors: Vec<Value> = wall.into_iter().map(|(name, given, msg, badge, last)| json!({
        "display_name": name,
        "total_given": given,
        "message": msg,
        "badge": if badge.is_empty() { json!(null) } else { json!(badge) },
        "last_donated_at": last,
    })).collect();

    Ok(Json(json!({ "donors": donors })))
}
