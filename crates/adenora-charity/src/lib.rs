pub mod campaign;
pub mod ledger;
pub mod project;

use adenora_common::currency::Currency;
use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A funded project or charity receiving proceeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharityProject {
    pub id: ProjectId,
    pub name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub country_codes: Vec<String>,
    pub logo_url: Option<String>,
    pub website_url: Option<String>,
    pub total_received: Decimal,
    pub currency: Currency,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCategory {
    DisasterRelief,
    Education,
    Healthcare,
    Infrastructure,
    CleanEnergy,
    WaterPurification,
    LandmineRemoval,
    Wildlife,
    Cultural,
    Sports,
    Other,
}

/// A single entry in the transparent charity ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub project_id: ProjectId,
    pub source: FundSource,
    pub amount: Decimal,
    pub currency: Currency,
    pub description: String,
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FundSource {
    /// Prediction market fee split (charity portion)
    PredictionFee,
    /// Lottery ticket revenue (project portion)
    LotteryRevenue,
    /// Tournament house cut (charity portion)
    TournamentFee,
    /// Direct donation to a cause campaign
    DirectDonation,
    /// H.E.L.P. game proceeds
    HelpGame,
    /// Purpose lottery — high-charity-pct lottery tied to a campaign
    PurposeLottery,
}

/// A purpose-driven fundraising campaign with a goal and deadline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseCampaign {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub tagline: String,
    pub description: String,
    pub category: CampaignCategory,
    pub goal_amount: Decimal,
    pub currency: Currency,
    pub amount_raised: Decimal,
    pub donor_count: i32,
    pub hero_image_url: Option<String>,
    pub impact_metric: Option<String>,
    pub impact_value: Option<String>,
    pub location: Option<String>,
    pub country_codes: Vec<String>,
    pub partner_org: Option<String>,
    pub status: CampaignStatus,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub funded_at: Option<DateTime<Utc>>,
    pub is_featured: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignCategory {
    Infrastructure,
    Water,
    Education,
    Healthcare,
    DisasterRelief,
    CleanEnergy,
    LandmineRemoval,
    Housing,
    FoodSecurity,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Active,
    Funded,
    Completed,
    Paused,
}

/// A direct donation from a user's wallet to a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectDonation {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub user_id: Option<UserId>,
    pub amount: Decimal,
    pub currency: Currency,
    pub message: Option<String>,
    pub is_anonymous: bool,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Summary of funds for a project — used in public dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFundingSummary {
    pub project_id: ProjectId,
    pub project_name: String,
    pub total_received: Decimal,
    pub from_predictions: Decimal,
    pub from_lottery: Decimal,
    pub from_tournaments: Decimal,
    pub from_donations: Decimal,
    pub from_help_games: Decimal,
    pub currency: Currency,
}
