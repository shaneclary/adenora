use rand::Rng;
use std::collections::HashSet;

/// Generate a set of unique random numbers for a lottery draw.
/// Uses cryptographically secure RNG.
pub fn generate_draw_numbers(
    count: usize,
    min: u32,
    max: u32,
) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut numbers = HashSet::new();

    while numbers.len() < count {
        numbers.insert(rng.gen_range(min..=max));
    }

    let mut result: Vec<u32> = numbers.into_iter().collect();
    result.sort();
    result
}

/// Check how many numbers match between a ticket and the draw.
pub fn count_matches(ticket_numbers: &[u32], draw_numbers: &[u32]) -> usize {
    let draw_set: HashSet<u32> = draw_numbers.iter().copied().collect();
    ticket_numbers
        .iter()
        .filter(|n| draw_set.contains(n))
        .count()
}

/// Determine prize tier based on number of matches.
pub fn prize_tier(matches: usize, total_numbers: usize) -> Option<super::PrizeTier> {
    use super::PrizeTier;

    if total_numbers == 0 {
        return None;
    }

    match (matches, total_numbers) {
        (m, t) if m == t => Some(PrizeTier::Jackpot),
        (m, t) if m == t - 1 => Some(PrizeTier::Second),
        (m, t) if m == t - 2 => Some(PrizeTier::Third),
        (m, _) if m >= 3 => Some(PrizeTier::Fourth),
        (m, _) if m >= 2 => Some(PrizeTier::Fifth),
        (1, _) => Some(PrizeTier::Free), // 1 match = free entry next draw
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_numbers_unique() {
        let numbers = generate_draw_numbers(6, 1, 49);
        assert_eq!(numbers.len(), 6);
        let set: HashSet<u32> = numbers.iter().copied().collect();
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn test_draw_numbers_in_range() {
        let numbers = generate_draw_numbers(6, 1, 49);
        for n in &numbers {
            assert!(*n >= 1 && *n <= 49);
        }
    }

    #[test]
    fn test_count_matches() {
        assert_eq!(count_matches(&[1, 2, 3], &[1, 2, 3, 4, 5, 6]), 3);
        assert_eq!(count_matches(&[1, 2, 3], &[4, 5, 6, 7, 8, 9]), 0);
        assert_eq!(count_matches(&[1, 2, 3, 4, 5, 6], &[1, 2, 3, 4, 5, 6]), 6);
    }

    #[test]
    fn test_prize_tier_jackpot() {
        assert!(matches!(prize_tier(6, 6), Some(crate::PrizeTier::Jackpot)));
    }
}
