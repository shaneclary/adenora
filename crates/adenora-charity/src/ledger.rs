use super::{FundSource, LedgerEntry};
use adenora_common::currency::Currency;
use adenora_common::types::ProjectId;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Record a fund allocation to a charity project.
pub fn record_allocation(
    project_id: ProjectId,
    source: FundSource,
    amount: Decimal,
    currency: Currency,
    description: &str,
    reference_id: Option<String>,
) -> LedgerEntry {
    LedgerEntry {
        id: Uuid::new_v4(),
        project_id,
        source,
        amount,
        currency,
        description: description.to_string(),
        reference_id,
        created_at: Utc::now(),
    }
}

/// Split a fee amount according to the standard distribution.
pub fn split_prediction_fee(
    total_fee: Decimal,
    project_id: ProjectId,
    currency: Currency,
    trade_ref: &str,
) -> Vec<LedgerEntry> {
    let charity_share = adenora_common::fees::split_fee(total_fee).charity;

    if charity_share > Decimal::ZERO {
        vec![record_allocation(
            project_id,
            FundSource::PredictionFee,
            charity_share,
            currency,
            &format!("prediction market fee split from trade {trade_ref}"),
            Some(trade_ref.to_string()),
        )]
    } else {
        Vec::new()
    }
}

/// Split lottery revenue according to game configuration.
pub fn split_lottery_revenue(
    ticket_price: Decimal,
    project_pct: u32,
    project_id: ProjectId,
    currency: Currency,
    ticket_ref: &str,
) -> LedgerEntry {
    let project_share = ticket_price * Decimal::from(project_pct) / Decimal::from(100);

    record_allocation(
        project_id,
        FundSource::LotteryRevenue,
        project_share,
        currency,
        &format!("lottery ticket revenue from {ticket_ref}"),
        Some(ticket_ref.to_string()),
    )
}
