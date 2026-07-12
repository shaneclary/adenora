use super::{CampaignStatus, CauseCampaign, DirectDonation};
use adenora_common::currency::Currency;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Compute progress percentage for a campaign (0–100, capped).
pub fn progress_pct(campaign: &CauseCampaign) -> u32 {
    if campaign.goal_amount == Decimal::ZERO {
        return 0;
    }
    let pct = (campaign.amount_raised / campaign.goal_amount * Decimal::from(100))
        .to_string()
        .parse::<f64>()
        .unwrap_or(0.0);
    pct.min(100.0) as u32
}

/// Determine the donor wall badge based on total given.
pub fn donor_badge(total: Decimal) -> &'static str {
    if total >= Decimal::from(10_000) { "Champion" }
    else if total >= Decimal::from(1_000) { "Founder" }
    else if total >= Decimal::from(100) { "Supporter" }
    else { "" }
}

/// Build a DirectDonation record (not persisted — caller does that).
pub fn build_donation(
    campaign_id: Uuid,
    user_id: Option<Uuid>,
    amount: Decimal,
    currency: Currency,
    message: Option<String>,
    is_anonymous: bool,
    display_name: Option<String>,
) -> DirectDonation {
    DirectDonation {
        id: Uuid::new_v4(),
        campaign_id,
        user_id,
        amount,
        currency,
        message,
        is_anonymous,
        display_name,
        created_at: Utc::now(),
    }
}

/// Check if a campaign is still accepting donations.
pub fn is_accepting(campaign: &CauseCampaign) -> bool {
    matches!(campaign.status, CampaignStatus::Active)
        && campaign.ends_at.is_none_or(|end| end > Utc::now())
}
