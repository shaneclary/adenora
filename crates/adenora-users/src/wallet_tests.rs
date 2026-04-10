use crate::wallet::Wallet;
use adenora_common::currency::Currency;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use uuid::Uuid;

fn test_wallet() -> Wallet {
    Wallet {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        currency: Currency::Eur,
        available: Decimal::from_str("100.00").unwrap(),
        reserved: Decimal::ZERO,
        updated_at: Utc::now(),
    }
}

#[test]
fn test_total_balance() {
    let mut w = test_wallet();
    w.reserved = Decimal::from_str("25.00").unwrap();
    assert_eq!(w.total(), Decimal::from_str("125.00").unwrap());
}

#[test]
fn test_can_afford() {
    let w = test_wallet();
    assert!(w.can_afford(Decimal::from_str("50.00").unwrap()));
    assert!(w.can_afford(Decimal::from_str("100.00").unwrap()));
    assert!(!w.can_afford(Decimal::from_str("100.01").unwrap()));
}

#[test]
fn test_reserve_funds() {
    let mut w = test_wallet();
    assert!(w.reserve(Decimal::from_str("30.00").unwrap()).is_ok());
    assert_eq!(w.available, Decimal::from_str("70.00").unwrap());
    assert_eq!(w.reserved, Decimal::from_str("30.00").unwrap());
}

#[test]
fn test_reserve_insufficient() {
    let mut w = test_wallet();
    assert!(w.reserve(Decimal::from_str("200.00").unwrap()).is_err());
    assert_eq!(w.available, Decimal::from_str("100.00").unwrap()); // unchanged
}

#[test]
fn test_release_funds() {
    let mut w = test_wallet();
    w.reserve(Decimal::from_str("50.00").unwrap()).unwrap();
    w.release(Decimal::from_str("20.00").unwrap());
    assert_eq!(w.available, Decimal::from_str("70.00").unwrap());
    assert_eq!(w.reserved, Decimal::from_str("30.00").unwrap());
}

#[test]
fn test_release_more_than_reserved() {
    let mut w = test_wallet();
    w.reserve(Decimal::from_str("10.00").unwrap()).unwrap();
    w.release(Decimal::from_str("50.00").unwrap()); // only 10 reserved
    assert_eq!(w.available, Decimal::from_str("100.00").unwrap());
    assert_eq!(w.reserved, Decimal::ZERO);
}

#[test]
fn test_credit() {
    let mut w = test_wallet();
    w.credit(Decimal::from_str("50.00").unwrap());
    assert_eq!(w.available, Decimal::from_str("150.00").unwrap());
}

#[test]
fn test_debit_reserved() {
    let mut w = test_wallet();
    w.reserve(Decimal::from_str("40.00").unwrap()).unwrap();
    w.debit_reserved(Decimal::from_str("25.00").unwrap());
    assert_eq!(w.reserved, Decimal::from_str("15.00").unwrap());
    assert_eq!(w.available, Decimal::from_str("60.00").unwrap()); // unchanged
}
