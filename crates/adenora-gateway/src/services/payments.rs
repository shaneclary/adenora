// Payment-provider interface scaffolding. The destination validators
// (`is_valid_iban`, `is_valid_eth_address`) are wired into the withdraw path;
// the payment-intent DTOs and constructors are retained for the planned SEPA/
// card/crypto rail integration and are not yet called.
#![allow(dead_code)]

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment gateway abstraction.
/// Supports SEPA instant transfers, card payments, and crypto (USDC on Polygon).
/// All deposits and withdrawals are FREE (0% fee).

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProvider {
    /// SEPA instant bank transfer (EUR) — primary for Kosovo, EU
    SepaInstant,
    /// Visa/Mastercard via payment processor
    Card,
    /// USDC on Polygon — crypto rail
    CryptoUsdc,
    /// Manual/bank wire
    BankWire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub direction: PaymentDirection,
    pub provider: PaymentProvider,
    pub amount: Decimal,
    pub currency: String,
    pub status: PaymentStatus,
    pub external_id: Option<String>,
    pub destination: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentDirection {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Create a deposit intent.
pub fn create_deposit(
    user_id: Uuid,
    provider: PaymentProvider,
    amount: Decimal,
    currency: &str,
) -> PaymentIntent {
    PaymentIntent {
        id: Uuid::new_v4(),
        user_id,
        direction: PaymentDirection::Deposit,
        provider,
        amount,
        currency: currency.to_string(),
        status: PaymentStatus::Pending,
        external_id: None,
        destination: None,
        created_at: chrono::Utc::now(),
    }
}

/// Create a withdrawal intent.
pub fn create_withdrawal(
    user_id: Uuid,
    provider: PaymentProvider,
    amount: Decimal,
    currency: &str,
    destination: &str,
) -> PaymentIntent {
    PaymentIntent {
        id: Uuid::new_v4(),
        user_id,
        direction: PaymentDirection::Withdrawal,
        provider,
        amount,
        currency: currency.to_string(),
        status: PaymentStatus::Pending,
        external_id: None,
        destination: Some(destination.to_string()),
        created_at: chrono::Utc::now(),
    }
}

/// Validate an IBAN: structural checks plus the ISO 13616 / ISO 7064 mod-97
/// checksum, so a structurally-plausible but invalid account number is rejected.
pub fn is_valid_iban(iban: &str) -> bool {
    let cleaned: String = iban
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if cleaned.len() < 15 || cleaned.len() > 34 {
        return false;
    }
    if !cleaned.is_char_boundary(4) {
        return false;
    }
    let (prefix, rest) = cleaned.split_at(4);
    if !prefix[..2].chars().all(|c| c.is_ascii_uppercase())
        || !prefix[2..].chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    if !rest.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    // Rearrange (move the first four chars to the end), map letters A..Z to
    // 10..35, then check the whole number is congruent to 1 modulo 97 — computed
    // digit-by-digit to avoid big integers.
    let rearranged = format!("{rest}{prefix}");
    let mut remainder: u32 = 0;
    for ch in rearranged.chars() {
        let value = if ch.is_ascii_digit() {
            ch as u32 - '0' as u32
        } else {
            ch as u32 - 'A' as u32 + 10
        };
        // Fold in one or two decimal digits at a time.
        if value >= 10 {
            remainder = (remainder * 100 + value) % 97;
        } else {
            remainder = (remainder * 10 + value) % 97;
        }
    }
    remainder == 1
}

/// Validate Ethereum/Polygon address format.
pub fn is_valid_eth_address(addr: &str) -> bool {
    addr.len() == 42
        && addr.starts_with("0x")
        && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_iban() {
        assert!(is_valid_iban("DE89370400440532013000"));
        assert!(is_valid_iban("AL35 2021 1109 0000 0000 0123 4567"));
        assert!(!is_valid_iban("short"));
        assert!(!is_valid_iban("1234567890123456"));
    }

    #[test]
    fn test_valid_eth_address() {
        assert!(is_valid_eth_address("0x1234567890abcdef1234567890abcdef12345678"));
        assert!(!is_valid_eth_address("0xshort"));
        assert!(!is_valid_eth_address("not-an-address"));
    }
}
