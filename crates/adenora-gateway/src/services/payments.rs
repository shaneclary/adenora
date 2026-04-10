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

/// Validate IBAN format (basic check for SEPA).
pub fn is_valid_iban(iban: &str) -> bool {
    let cleaned: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
    cleaned.len() >= 15
        && cleaned.len() <= 34
        && cleaned[..2].chars().all(|c| c.is_ascii_uppercase())
        && cleaned[2..4].chars().all(|c| c.is_ascii_digit())
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
