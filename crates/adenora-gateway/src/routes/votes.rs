use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
fn err(s: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (s, Json(json!({"error": msg})))
}

/// Row for a vote campaign list entry.
type VoteCampaignRow = (
    Uuid, String, String, String, String, String, i32,
    Decimal, String, String,
    chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>,
);

/// Row for a single vote campaign's core fields (incl. winning proposal).
type VoteCampaignCoreRow = (
    Uuid, String, String, String, String, String, i32,
    Decimal, String, String,
    chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>,
    Option<Uuid>,
);

/// Row for a vote proposal with tallies.
type VoteProposalRow = (Uuid, String, String, Option<String>, i32, Decimal, i32);

// ─── List vote campaigns ────────────────────────────────────────────────────

/// GET /api/v1/votes — list active vote campaigns
pub async fn list_campaigns(State(state): State<AppState>) -> ApiResult {
    let rows: Vec<VoteCampaignRow> = sqlx::query_as(
        "SELECT id, title, description, category, vote_mode, fund_mode, vote_cap,
                seed_amount, currency, status, opens_at, closes_at
         FROM vote_campaigns
         WHERE status IN ('open', 'voting')
         ORDER BY closes_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let now = chrono::Utc::now();
    let campaigns: Vec<Value> = rows.into_iter().map(|(id, title, desc, cat, vote_mode, fund_mode, vote_cap, seed, currency, status, _opens_at, closes_at)| {
        let remaining = closes_at - now;
        let time_left = if remaining.num_days() > 0 {
            format!("{}d", remaining.num_days())
        } else if remaining.num_hours() > 0 {
            format!("{}h", remaining.num_hours())
        } else { "soon".to_string() };

        json!({
            "id": id, "title": title, "description": desc, "category": cat,
            "vote_mode": vote_mode, "fund_mode": fund_mode, "vote_cap": vote_cap,
            "seed_amount": seed, "currency": currency, "status": status,
            "time_left": time_left, "closes_at": closes_at,
        })
    }).collect();

    Ok(Json(json!({ "campaigns": campaigns, "total": campaigns.len() })))
}

// ─── Get single campaign with proposals + results ───────────────────────────

/// GET /api/v1/votes/{id} — campaign detail with proposals and vote tallies
pub async fn get_campaign(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult {
    // Split into two queries to stay within sqlx 16-column tuple limit
    let core: Option<VoteCampaignCoreRow> = sqlx::query_as(
        "SELECT id, title, description, category, vote_mode, fund_mode, vote_cap,
                seed_amount, currency, status, opens_at, closes_at, winning_proposal_id
         FROM vote_campaigns WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (cid, title, desc, cat, vote_mode, fund_mode, vote_cap,
         seed, currency, status, _opens_at, closes_at, winning_id) = core
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "campaign not found"))?;

    let extras: Option<(i32, i32, i32, Option<String>)> = sqlx::query_as(
        "SELECT vc.winner_pct, vc.charity_pct, vc.pool_pct, cp.name
         FROM vote_campaigns vc
         LEFT JOIN charity_projects cp ON vc.charity_project_id = cp.id
         WHERE vc.id = $1"
    )
    .bind(cid)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (winner_pct, charity_pct, pool_pct, charity_name) =
        extras.unwrap_or((60, 20, 20, None));

    // Get proposals with tallies
    let proposals: Vec<VoteProposalRow> = sqlx::query_as(
        "SELECT id, title, description, image_url, vote_count, fund_total, voter_count
         FROM vote_proposals WHERE campaign_id = $1
         ORDER BY vote_count DESC, fund_total DESC"
    )
    .bind(cid)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let total_votes: i32 = proposals.iter().map(|p| p.4).sum();
    let total_funds: Decimal = proposals.iter().map(|p| p.5).sum();

    let proposals_json: Vec<Value> = proposals.iter().map(|(pid, title, desc, img, votes, funds, voters)| {
        let vote_pct = if total_votes > 0 { (*votes as f64 / total_votes as f64 * 100.0) as u32 } else { 0 };
        json!({
            "id": pid, "title": title, "description": desc, "image_url": img,
            "vote_count": votes, "fund_total": funds.round_dp(2), "voter_count": voters,
            "vote_pct": vote_pct,
            "is_winner": winning_id.as_ref() == Some(pid),
        })
    }).collect();

    // Public conviction feed (latest votes with comments)
    let feed: Vec<(String, String, Decimal, i32, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT u.display_name, vp.title, cv.amount_eur, cv.votes_counted, cv.comment, cv.created_at
         FROM community_votes cv
         JOIN users u ON u.id = cv.user_id
         JOIN vote_proposals vp ON vp.id = cv.proposal_id
         WHERE cv.campaign_id = $1 AND cv.is_public = true
         ORDER BY cv.created_at DESC LIMIT 30"
    )
    .bind(cid)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let feed_json: Vec<Value> = feed.into_iter().map(|(name, proposal, amount, votes, comment, at)| json!({
        "name": name, "proposal": proposal,
        "amount_eur": amount.round_dp(2), "votes": votes,
        "comment": if comment.is_empty() { None } else { Some(comment) },
        "at": at,
    })).collect();

    let now = chrono::Utc::now();
    let remaining = closes_at - now;
    let time_left = if remaining.num_days() > 0 {
        format!("{} days", remaining.num_days())
    } else if remaining.num_hours() > 0 {
        format!("{} hours", remaining.num_hours())
    } else { "closing soon".to_string() };

    Ok(Json(json!({
        "id": cid, "title": title, "description": desc, "category": cat,
        "vote_mode": vote_mode, "fund_mode": fund_mode, "vote_cap": vote_cap,
        "split": { "winner_pct": winner_pct, "charity_pct": charity_pct, "pool_pct": pool_pct },
        "seed_amount": seed, "currency": currency, "status": status,
        "total_votes": total_votes, "total_funds": (total_funds + seed).round_dp(2),
        "time_left": time_left, "closes_at": closes_at,
        "charity_project": charity_name,
        "winning_proposal_id": winning_id,
        "proposals": proposals_json,
        "feed": feed_json,
    })))
}

// ─── Cast a vote ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CastVoteRequest {
    pub proposal_id: Uuid,
    pub amount_eur: Decimal,
    pub comment: Option<String>,
    pub is_public: Option<bool>,
}

/// POST /api/v1/votes/{id}/vote — cast your vote on a proposal
pub async fn cast_vote(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(campaign_id): Path<Uuid>,
    Json(req): Json<CastVoteRequest>,
) -> ApiResult {
    if req.amount_eur <= Decimal::ZERO {
        return Err(err(StatusCode::BAD_REQUEST, "amount must be positive"));
    }

    // KYC check (required for one_person mode, recommended for all)
    let user: Option<(String,)> = sqlx::query_as(
        "SELECT kyc_status FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Load campaign config
    let campaign: Option<(String, String, i32, String)> = sqlx::query_as(
        "SELECT status, vote_mode, vote_cap, currency FROM vote_campaigns WHERE id = $1"
    )
    .bind(campaign_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (status, vote_mode, vote_cap, currency) = campaign
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "campaign not found"))?;

    if status != "open" && status != "voting" {
        return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "campaign is not accepting votes"));
    }

    // Enforce one_person mode requires KYC
    if vote_mode == "one_person" {
        match user.as_ref().map(|(s,)| s.as_str()) {
            Some("verified") | Some("approved") => {}
            _ => return Err(err(StatusCode::FORBIDDEN, "KYC verification required for one-person-one-vote campaigns")),
        }
    }

    // Verify proposal belongs to this campaign
    let proposal_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM vote_proposals WHERE id = $1 AND campaign_id = $2"
    )
    .bind(req.proposal_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if proposal_check.is_none() {
        return Err(err(StatusCode::BAD_REQUEST, "proposal does not belong to this campaign"));
    }

    // Calculate votes_counted based on mode
    let votes_counted: i32 = match vote_mode.as_str() {
        "one_person" => 1,
        "capped" => {
            // €1 = 1 vote, capped at vote_cap
            let raw = req.amount_eur.floor().to_string().parse::<i32>().unwrap_or(1);
            raw.min(vote_cap)
        }
        "uncapped" => {
            req.amount_eur.floor().to_string().parse::<i32>().unwrap_or(1)
        }
        _ => 1,
    };

    // Debit wallet
    let debited: Option<(Decimal,)> = sqlx::query_as(
        "UPDATE wallets SET available = available - $1, updated_at = NOW()
         WHERE user_id = $2 AND currency = $3 AND available >= $1
         RETURNING available"
    )
    .bind(req.amount_eur)
    .bind(auth.user_id)
    .bind(&currency)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if debited.is_none() {
        return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient balance"));
    }

    // Sanitize comment
    let comment = req.comment.as_deref().unwrap_or("");
    let safe_comment: String = comment.chars()
        .filter(|c| *c != '<' && *c != '>')
        .take(500)
        .collect();
    let is_public = req.is_public.unwrap_or(false);

    // Upsert vote (one vote per user per campaign — changing your vote moves money)
    let existing: Option<(Uuid, Uuid, Decimal)> = sqlx::query_as(
        "SELECT id, proposal_id, amount_eur FROM community_votes
         WHERE campaign_id = $1 AND user_id = $2"
    )
    .bind(campaign_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if let Some((_vid, _old_proposal, old_amount)) = existing {
        // Update existing vote — add more funds
        sqlx::query(
            "UPDATE community_votes SET
                proposal_id = $1, amount_eur = amount_eur + $2,
                votes_counted = $3, comment = $4, is_public = $5, updated_at = NOW()
             WHERE campaign_id = $6 AND user_id = $7"
        )
        .bind(req.proposal_id)
        .bind(req.amount_eur)
        .bind(votes_counted)
        .bind(&safe_comment)
        .bind(is_public)
        .bind(campaign_id)
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        // Recalculate votes_counted for the updated total
        if vote_mode == "capped" {
            let new_total = old_amount + req.amount_eur;
            let new_votes = new_total.floor().to_string().parse::<i32>().unwrap_or(1).min(vote_cap);
            sqlx::query(
                "UPDATE community_votes SET votes_counted = $1 WHERE campaign_id = $2 AND user_id = $3"
            )
            .bind(new_votes)
            .bind(campaign_id)
            .bind(auth.user_id)
            .execute(&state.db)
            .await
            .ok();
        }
    } else {
        // New vote
        sqlx::query(
            "INSERT INTO community_votes (campaign_id, proposal_id, user_id, amount_eur, votes_counted, comment, is_public)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(campaign_id)
        .bind(req.proposal_id)
        .bind(auth.user_id)
        .bind(req.amount_eur)
        .bind(votes_counted)
        .bind(&safe_comment)
        .bind(is_public)
        .execute(&state.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    }

    // Record transaction
    sqlx::query(
        "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
         VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = $2 LIMIT 1),
                 'vote', $3, $2, 'vote_campaign', $4, $5)"
    )
    .bind(auth.user_id)
    .bind(&currency)
    .bind(req.amount_eur)
    .bind(campaign_id)
    .bind(format!("vote on campaign {}", campaign_id))
    .execute(&state.db)
    .await
    .ok();

    // Get display name
    let name: String = sqlx::query_as::<_, (String,)>(
        "SELECT display_name FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|(n,)| n)
    .unwrap_or_else(|| "Anonymous".to_string());

    tracing::info!(
        user_id = %auth.user_id,
        campaign_id = %campaign_id,
        proposal_id = %req.proposal_id,
        amount = %req.amount_eur,
        votes = votes_counted,
        "vote cast"
    );

    Ok(Json(json!({
        "status": "voted",
        "campaign_id": campaign_id,
        "proposal_id": req.proposal_id,
        "amount_eur": req.amount_eur,
        "votes_counted": votes_counted,
        "vote_mode": vote_mode,
        "vote_cap": vote_cap,
        "is_public": is_public,
        "message": format!("{} voted with {} conviction!", name, votes_counted),
    })))
}

// ─── Create a vote campaign (sponsor) ───────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateCampaignRequest {
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub vote_mode: String,
    pub vote_cap: Option<i32>,
    pub fund_mode: String,
    pub winner_pct: Option<i32>,
    pub charity_pct: Option<i32>,
    pub pool_pct: Option<i32>,
    pub seed_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub closes_at: String,
    pub proposals: Vec<ProposalInput>,
    pub charity_project_id: Option<Uuid>,
    pub community_pool_id: Option<Uuid>,
    pub country_codes: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ProposalInput {
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

/// POST /api/v1/votes — create a new vote campaign
pub async fn create_campaign(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateCampaignRequest>,
) -> ApiResult {
    // Validate
    if req.title.trim().is_empty() || req.title.len() > 300 {
        return Err(err(StatusCode::BAD_REQUEST, "title must be 1-300 characters"));
    }
    if req.proposals.is_empty() || req.proposals.len() > 20 {
        return Err(err(StatusCode::BAD_REQUEST, "must have 1-20 proposals"));
    }

    let vote_mode = match req.vote_mode.as_str() {
        "one_person" | "capped" | "uncapped" => &req.vote_mode,
        _ => return Err(err(StatusCode::BAD_REQUEST, "vote_mode must be 'one_person', 'capped', or 'uncapped'")),
    };
    let fund_mode = match req.fund_mode.as_str() {
        "winner_take_all" | "winner_impact" | "winner_pool" => &req.fund_mode,
        _ => return Err(err(StatusCode::BAD_REQUEST, "fund_mode must be 'winner_take_all', 'winner_impact', or 'winner_pool'")),
    };

    // Split percentages
    let (w, c, p) = match fund_mode.as_str() {
        "winner_take_all" => (100, 0, 0),
        _ => {
            let w = req.winner_pct.unwrap_or(60);
            let c = req.charity_pct.unwrap_or(20);
            let p = req.pool_pct.unwrap_or(20);
            if w + c + p != 100 {
                return Err(err(StatusCode::BAD_REQUEST, "winner_pct + charity_pct + pool_pct must equal 100"));
            }
            (w, c, p)
        }
    };

    let closes_at = chrono::DateTime::parse_from_rfc3339(&req.closes_at)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "closes_at must be valid ISO 8601"))?
        .with_timezone(&chrono::Utc);

    let seed = req.seed_amount.unwrap_or(Decimal::ZERO);
    let currency = req.currency.as_deref().unwrap_or("EUR");
    let category = req.category.as_deref().unwrap_or("community");
    let vote_cap = req.vote_cap.unwrap_or(100);

    // Debit seed from creator wallet if > 0
    if seed > Decimal::ZERO {
        let debited: Option<(Decimal,)> = sqlx::query_as(
            "UPDATE wallets SET available = available - $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = $3 AND available >= $1
             RETURNING available"
        )
        .bind(seed)
        .bind(auth.user_id)
        .bind(currency)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        if debited.is_none() {
            return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient balance for seed amount"));
        }
    }

    // Create campaign
    let campaign_id: (Uuid,) = sqlx::query_as(
        "INSERT INTO vote_campaigns
            (creator_id, title, description, category, vote_mode, vote_cap,
             fund_mode, winner_pct, charity_pct, pool_pct,
             seed_amount, currency, closes_at, status,
             charity_project_id, community_pool_id, country_codes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'open', $14, $15, $16)
         RETURNING id"
    )
    .bind(auth.user_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(category)
    .bind(vote_mode)
    .bind(vote_cap)
    .bind(fund_mode)
    .bind(w).bind(c).bind(p)
    .bind(seed)
    .bind(currency)
    .bind(closes_at)
    .bind(req.charity_project_id)
    .bind(req.community_pool_id)
    .bind(req.country_codes.as_deref().unwrap_or(&[]))
    .fetch_one(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Create proposals
    for (i, prop) in req.proposals.iter().enumerate() {
        sqlx::query(
            "INSERT INTO vote_proposals (campaign_id, title, description, image_url, proposer_id, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(campaign_id.0)
        .bind(&prop.title)
        .bind(prop.description.as_deref().unwrap_or(""))
        .bind(&prop.image_url)
        .bind(auth.user_id)
        .bind(i as i32)
        .execute(&state.db)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    }

    tracing::info!(
        creator_id = %auth.user_id,
        campaign_id = %campaign_id.0,
        title = %req.title,
        proposals = req.proposals.len(),
        vote_mode = %vote_mode,
        fund_mode = %fund_mode,
        seed = %seed,
        "vote campaign created"
    );

    Ok(Json(json!({
        "campaign_id": campaign_id.0,
        "title": req.title,
        "proposals": req.proposals.len(),
        "vote_mode": vote_mode,
        "fund_mode": fund_mode,
        "closes_at": closes_at,
        "message": "Campaign created. Voting is open!",
    })))
}
