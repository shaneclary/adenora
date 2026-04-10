use super::{CharityProject, ProjectCategory};
use adenora_common::currency::Currency;
use adenora_common::types::ProjectId;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

pub fn create_project(
    name: String,
    description: String,
    category: ProjectCategory,
    country_codes: Vec<String>,
    currency: Currency,
) -> CharityProject {
    CharityProject {
        id: Uuid::new_v4(),
        name,
        description,
        category,
        country_codes,
        logo_url: None,
        website_url: None,
        total_received: Decimal::ZERO,
        currency,
        is_active: true,
        created_at: Utc::now(),
    }
}
