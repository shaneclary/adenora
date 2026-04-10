use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Check all responsible gambling restrictions for a user before allowing activity.
pub async fn check_gambling_restrictions(
    db: &PgPool,
    user_id: Uuid,
) -> Result<(), GamblingRestriction> {
    let row: Option<(
        Option<DateTime<Utc>>,  // self_exclusion_until
        Option<DateTime<Utc>>,  // cooling_off_until
    )> = sqlx::query_as(
        "SELECT self_exclusion_until, cooling_off_until FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|_| GamblingRestriction::SystemError)?;

    let (self_exclusion, cooling_off) = row
        .ok_or(GamblingRestriction::SystemError)?;

    let now = Utc::now();

    if let Some(until) = self_exclusion {
        if now < until {
            return Err(GamblingRestriction::SelfExcluded { until });
        }
    }

    if let Some(until) = cooling_off {
        if now < until {
            return Err(GamblingRestriction::CoolingOff { until });
        }
    }

    Ok(())
}

/// Check deposit limits.
pub async fn check_deposit_limits(
    db: &PgPool,
    user_id: Uuid,
    amount: Decimal,
) -> Result<(), GamblingRestriction> {
    let limits: Option<(Option<Decimal>, Option<Decimal>, Option<Decimal>)> = sqlx::query_as(
        "SELECT daily_deposit_limit, weekly_deposit_limit, monthly_deposit_limit
         FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|_| GamblingRestriction::SystemError)?;

    let (daily, weekly, monthly) = limits.ok_or(GamblingRestriction::SystemError)?;

    if let Some(limit) = daily {
        let today: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE user_id = $1 AND tx_type = 'deposit' AND created_at >= CURRENT_DATE"
        )
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|_| GamblingRestriction::SystemError)?;

        if today.0 + amount > limit {
            return Err(GamblingRestriction::DailyLimitExceeded {
                limit,
                used: today.0,
            });
        }
    }

    if let Some(limit) = weekly {
        let week: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE user_id = $1 AND tx_type = 'deposit'
             AND created_at >= CURRENT_DATE - INTERVAL '7 days'"
        )
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|_| GamblingRestriction::SystemError)?;

        if week.0 + amount > limit {
            return Err(GamblingRestriction::WeeklyLimitExceeded {
                limit,
                used: week.0,
            });
        }
    }

    if let Some(limit) = monthly {
        let month: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE user_id = $1 AND tx_type = 'deposit'
             AND created_at >= CURRENT_DATE - INTERVAL '30 days'"
        )
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|_| GamblingRestriction::SystemError)?;

        if month.0 + amount > limit {
            return Err(GamblingRestriction::MonthlyLimitExceeded {
                limit,
                used: month.0,
            });
        }
    }

    Ok(())
}

/// Trigger cooling-off period after significant losses.
/// Called after each trade settlement.
pub async fn check_loss_alert(
    db: &PgPool,
    user_id: Uuid,
) -> Result<(), GamblingRestriction> {
    let limits: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT loss_alert_threshold FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
    .map_err(|_| GamblingRestriction::SystemError)?;

    let (threshold,) = limits.ok_or(GamblingRestriction::SystemError)?;

    if let Some(threshold) = threshold {
        // Calculate recent losses (last 24h)
        let losses: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(CASE WHEN amount < 0 THEN ABS(amount) ELSE 0 END), 0)
             FROM transactions
             WHERE user_id = $1 AND tx_type IN ('trade_buy', 'trade_sell', 'fee')
             AND created_at >= NOW() - INTERVAL '24 hours'"
        )
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|_| GamblingRestriction::SystemError)?;

        if losses.0 >= threshold {
            // Auto-trigger 1 hour cooling-off
            let until = Utc::now() + Duration::hours(1);
            sqlx::query("UPDATE users SET cooling_off_until = $1 WHERE id = $2")
                .bind(until)
                .bind(user_id)
                .execute(db)
                .await
                .ok();

            return Err(GamblingRestriction::LossAlertTriggered {
                losses: losses.0,
                threshold,
                cooling_off_until: until,
            });
        }
    }

    Ok(())
}

/// Set self-exclusion for a user.
pub async fn set_self_exclusion(
    db: &PgPool,
    user_id: Uuid,
    days: u32,
) -> Result<DateTime<Utc>, GamblingRestriction> {
    let until = Utc::now() + Duration::days(days as i64);

    sqlx::query("UPDATE users SET self_exclusion_until = $1, updated_at = NOW() WHERE id = $2")
        .bind(until)
        .bind(user_id)
        .execute(db)
        .await
        .map_err(|_| GamblingRestriction::SystemError)?;

    Ok(until)
}

#[derive(Debug)]
pub enum GamblingRestriction {
    SelfExcluded { until: DateTime<Utc> },
    CoolingOff { until: DateTime<Utc> },
    DailyLimitExceeded { limit: Decimal, used: Decimal },
    WeeklyLimitExceeded { limit: Decimal, used: Decimal },
    MonthlyLimitExceeded { limit: Decimal, used: Decimal },
    LossAlertTriggered {
        losses: Decimal,
        threshold: Decimal,
        cooling_off_until: DateTime<Utc>,
    },
    SystemError,
}

impl std::fmt::Display for GamblingRestriction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfExcluded { until } => write!(f, "self-exclusion active until {}", until),
            Self::CoolingOff { until } => write!(f, "cooling-off period active until {}", until),
            Self::DailyLimitExceeded { limit, used } => write!(f, "daily deposit limit ({}) exceeded, used {}", limit, used),
            Self::WeeklyLimitExceeded { limit, used } => write!(f, "weekly deposit limit ({}) exceeded, used {}", limit, used),
            Self::MonthlyLimitExceeded { limit, used } => write!(f, "monthly deposit limit ({}) exceeded, used {}", limit, used),
            Self::LossAlertTriggered { losses, threshold, cooling_off_until } =>
                write!(f, "loss alert: {} losses exceeded {} threshold, cooling off until {}", losses, threshold, cooling_off_until),
            Self::SystemError => write!(f, "system error checking gambling restrictions"),
        }
    }
}
