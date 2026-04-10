use super::GamblingLimits;
use adenora_common::error::AdenoraError;
use chrono::Utc;
use rust_decimal::Decimal;

pub fn check_deposit_allowed(
    limits: &GamblingLimits,
    amount: Decimal,
    deposits_today: Decimal,
    deposits_this_week: Decimal,
    deposits_this_month: Decimal,
) -> Result<(), AdenoraError> {
    // Check self-exclusion
    if let Some(until) = limits.self_exclusion_until {
        if Utc::now() < until {
            return Err(AdenoraError::SelfExclusionActive(until.to_string()));
        }
    }

    // Check cooling-off period
    if let Some(until) = limits.cooling_off_until {
        if Utc::now() < until {
            return Err(AdenoraError::CoolingOffActive);
        }
    }

    // Check deposit limits
    if let Some(daily) = limits.daily_deposit_limit {
        if deposits_today + amount > daily {
            return Err(AdenoraError::DepositLimitExceeded(format!(
                "daily limit {} exceeded",
                daily
            )));
        }
    }

    if let Some(weekly) = limits.weekly_deposit_limit {
        if deposits_this_week + amount > weekly {
            return Err(AdenoraError::DepositLimitExceeded(format!(
                "weekly limit {} exceeded",
                weekly
            )));
        }
    }

    if let Some(monthly) = limits.monthly_deposit_limit {
        if deposits_this_month + amount > monthly {
            return Err(AdenoraError::DepositLimitExceeded(format!(
                "monthly limit {} exceeded",
                monthly
            )));
        }
    }

    Ok(())
}
