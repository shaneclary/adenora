pub mod auth;
pub mod kyc;
pub mod limits;
pub mod wallet;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wallet_tests;

use adenora_common::currency::Currency;
use adenora_common::i18n::Locale;
use adenora_common::types::*;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub display_name: String,
    pub locale: Locale,
    pub preferred_currency: Currency,
    pub kyc_status: KycStatus,
    pub date_of_birth: Option<NaiveDate>,
    pub country_code: String,
    pub is_bot_account: bool,
    pub is_admin: bool,
    pub gambling_limits: GamblingLimits,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KycStatus {
    None,
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GamblingLimits {
    pub daily_deposit_limit: Option<Decimal>,
    pub weekly_deposit_limit: Option<Decimal>,
    pub monthly_deposit_limit: Option<Decimal>,
    pub self_exclusion_until: Option<DateTime<Utc>>,
    pub cooling_off_until: Option<DateTime<Utc>>,
    pub loss_alert_threshold: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub locale: Option<Locale>,
    pub preferred_currency: Option<Currency>,
    pub date_of_birth: NaiveDate,
    pub country_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}
