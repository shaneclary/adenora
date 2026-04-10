use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Kalshi-matched parabolic fee schedule.
///
/// Taker: fee = ceil(TAKER_COEFF * contracts * P * (1 - P))
/// Maker: fee = ceil(MAKER_COEFF * contracts * P * (1 - P))
///
/// Where P is the contract price in dollars (0.0 to 1.0).
/// Fees are ceiled to the nearest centicent ($0.0001).
/// Maximum taker fee at P=0.50: ~1.75 cents per contract.
/// Maximum maker fee at P=0.50: ~0.44 cents per contract.

const TAKER_COEFF: &str = "0.07";
const MAKER_COEFF: &str = "0.0175";
const CENTICENT: &str = "0.0001";

/// Fee split percentages
const OPS_SPLIT_PCT: u32 = 40;
const CHARITY_SPLIT_PCT: u32 = 40;
const CREATOR_SPLIT_PCT: u32 = 20;

#[derive(Debug, Clone, Copy)]
pub struct FeeSplit {
    pub total: Decimal,
    pub ops: Decimal,
    pub charity: Decimal,
    pub creator: Decimal,
}

pub fn calculate_taker_fee(contracts: u32, price_dollars: Decimal) -> Decimal {
    calculate_fee(contracts, price_dollars, Decimal::from_str(TAKER_COEFF).unwrap())
}

pub fn calculate_maker_fee(contracts: u32, price_dollars: Decimal) -> Decimal {
    calculate_fee(contracts, price_dollars, Decimal::from_str(MAKER_COEFF).unwrap())
}

fn calculate_fee(contracts: u32, price_dollars: Decimal, coeff: Decimal) -> Decimal {
    let one = Decimal::ONE;
    let centicent = Decimal::from_str(CENTICENT).unwrap();

    let raw = coeff * Decimal::from(contracts) * price_dollars * (one - price_dollars);

    // Ceil to nearest centicent
    (raw / centicent).ceil() * centicent
}

pub fn split_fee(total_fee: Decimal) -> FeeSplit {
    let hundred = Decimal::from(100);
    FeeSplit {
        total: total_fee,
        ops: total_fee * Decimal::from(OPS_SPLIT_PCT) / hundred,
        charity: total_fee * Decimal::from(CHARITY_SPLIT_PCT) / hundred,
        creator: total_fee * Decimal::from(CREATOR_SPLIT_PCT) / hundred,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taker_fee_at_midpoint() {
        // At P=0.50, fee should be ~1.75 cents per contract
        let fee = calculate_taker_fee(1, Decimal::from_str("0.50").unwrap());
        assert_eq!(fee, Decimal::from_str("0.0175").unwrap());
    }

    #[test]
    fn test_taker_fee_at_extremes() {
        // At P=0.05, fee should be ~0.33 cents
        let fee = calculate_taker_fee(1, Decimal::from_str("0.05").unwrap());
        assert!(fee < Decimal::from_str("0.0040").unwrap());
    }

    #[test]
    fn test_maker_fee_is_approximately_quarter_of_taker() {
        let taker = calculate_taker_fee(1, Decimal::from_str("0.50").unwrap());
        let maker = calculate_maker_fee(1, Decimal::from_str("0.50").unwrap());
        // Maker coefficient (0.0175) is 1/4 of taker (0.07), but ceiling
        // rounding means the result may differ by up to 1 centicent
        let quarter = taker / Decimal::from(4);
        let diff = (maker - quarter).abs();
        assert!(diff <= Decimal::from_str("0.0001").unwrap());
    }

    #[test]
    fn test_fee_split() {
        let split = split_fee(Decimal::from_str("1.00").unwrap());
        assert_eq!(split.ops, Decimal::from_str("0.40").unwrap());
        assert_eq!(split.charity, Decimal::from_str("0.40").unwrap());
        assert_eq!(split.creator, Decimal::from_str("0.20").unwrap());
    }

    #[test]
    fn test_multiple_contracts() {
        let fee_1 = calculate_taker_fee(1, Decimal::from_str("0.50").unwrap());
        let fee_5 = calculate_taker_fee(5, Decimal::from_str("0.50").unwrap());
        assert!(fee_5 >= fee_1 * Decimal::from(5));
    }

    #[test]
    fn test_fee_is_zero_at_boundaries() {
        // At P=0 and P=1, fee should be 0 (P*(1-P) = 0)
        let fee_0 = calculate_taker_fee(1, Decimal::ZERO);
        let fee_1 = calculate_taker_fee(1, Decimal::ONE);
        assert_eq!(fee_0, Decimal::ZERO);
        assert_eq!(fee_1, Decimal::ZERO);
    }

    #[test]
    fn test_fee_symmetry() {
        // Fee at P=0.30 should equal fee at P=0.70 (parabolic is symmetric)
        let fee_30 = calculate_taker_fee(1, Decimal::from_str("0.30").unwrap());
        let fee_70 = calculate_taker_fee(1, Decimal::from_str("0.70").unwrap());
        assert_eq!(fee_30, fee_70);
    }

    #[test]
    fn test_fee_maximum_at_midpoint() {
        // The parabolic peak is at P=0.50
        let fee_50 = calculate_taker_fee(1, Decimal::from_str("0.50").unwrap());
        let fee_30 = calculate_taker_fee(1, Decimal::from_str("0.30").unwrap());
        let fee_70 = calculate_taker_fee(1, Decimal::from_str("0.70").unwrap());
        let fee_10 = calculate_taker_fee(1, Decimal::from_str("0.10").unwrap());
        assert!(fee_50 >= fee_30);
        assert!(fee_50 >= fee_70);
        assert!(fee_50 >= fee_10);
    }

    #[test]
    fn test_fee_always_non_negative() {
        for cents in 0..=100 {
            let p = Decimal::new(cents, 2);
            let fee = calculate_taker_fee(1, p);
            assert!(fee >= Decimal::ZERO, "fee negative at p={}", p);
        }
    }

    #[test]
    fn test_split_adds_up() {
        let total = Decimal::from_str("10.00").unwrap();
        let split = split_fee(total);
        assert_eq!(split.ops + split.charity + split.creator, total);
    }

    #[test]
    fn test_zero_fee_split() {
        let split = split_fee(Decimal::ZERO);
        assert_eq!(split.total, Decimal::ZERO);
        assert_eq!(split.ops, Decimal::ZERO);
        assert_eq!(split.charity, Decimal::ZERO);
        assert_eq!(split.creator, Decimal::ZERO);
    }
}
