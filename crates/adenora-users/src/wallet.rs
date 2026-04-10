use adenora_common::currency::Currency;
use adenora_common::error::AdenoraError;
use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: UserId,
    pub currency: Currency,
    pub available: Decimal,
    pub reserved: Decimal, // locked in open orders / positions
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    pub fn total(&self) -> Decimal {
        self.available + self.reserved
    }

    pub fn can_afford(&self, amount: Decimal) -> bool {
        self.available >= amount
    }

    pub fn reserve(&mut self, amount: Decimal) -> Result<(), AdenoraError> {
        if self.available < amount {
            return Err(AdenoraError::InsufficientBalance {
                need: amount.to_string(),
                have: self.available.to_string(),
            });
        }
        self.available -= amount;
        self.reserved += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn release(&mut self, amount: Decimal) {
        let release = amount.min(self.reserved);
        self.reserved -= release;
        self.available += release;
        self.updated_at = Utc::now();
    }

    pub fn credit(&mut self, amount: Decimal) {
        self.available += amount;
        self.updated_at = Utc::now();
    }

    pub fn debit_reserved(&mut self, amount: Decimal) {
        let debit = amount.min(self.reserved);
        self.reserved -= debit;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    pub currency: Currency,
    pub amount: Decimal,
    pub method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    pub currency: Currency,
    pub amount: Decimal,
    pub method: PaymentMethod,
    pub destination: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    BankTransfer,
    Card,
    Crypto,
    MobileTopup,
}
