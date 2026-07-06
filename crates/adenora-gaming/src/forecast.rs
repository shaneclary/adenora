use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A forecasting challenge — predict probabilities for upcoming events.
/// Scored using Brier score (lower is better, 0 = perfect).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastChallenge {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub questions: Vec<ForecastQuestion>,
    pub status: ChallengeStatus,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStatus {
    Upcoming,
    Active,
    Scoring,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastQuestion {
    pub id: Uuid,
    pub question: String,
    pub resolved: bool,
    pub actual_outcome: Option<bool>, // true = happened, false = didn't
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPrediction {
    pub user_id: Uuid,
    pub question_id: Uuid,
    pub probability: f64, // 0.0 to 1.0
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub user_id: Uuid,
    pub challenge_id: Uuid,
    pub brier_score: f64,
    pub rank: u32,
    pub questions_answered: u32,
    pub calibration: f64,
}

/// Calculate Brier score for a set of predictions.
/// Brier score = mean of (predicted_probability - actual_outcome)^2
/// Range: 0 (perfect) to 1 (worst possible).
/// Score of 0.25 = random guessing on binary events.
pub fn brier_score(predictions: &[(f64, bool)]) -> f64 {
    if predictions.is_empty() {
        return 1.0;
    }

    let sum: f64 = predictions
        .iter()
        .map(|(prob, outcome)| {
            let actual = if *outcome { 1.0 } else { 0.0 };
            (prob - actual).powi(2)
        })
        .sum();

    sum / predictions.len() as f64
}

/// Calculate calibration score.
/// Groups predictions into buckets (0-10%, 10-20%, ...) and measures
/// how close the actual frequency is to the predicted probability.
/// Perfect calibration = 1.0, worst = 0.0.
pub fn calibration_score(predictions: &[(f64, bool)]) -> f64 {
    if predictions.len() < 10 {
        return 0.5; // Not enough data
    }

    let buckets = 10;
    let mut bucket_counts = vec![0u32; buckets];
    let mut bucket_correct = vec![0u32; buckets];

    for (prob, outcome) in predictions {
        let bucket = ((*prob * buckets as f64) as usize).min(buckets - 1);
        bucket_counts[bucket] += 1;
        if *outcome {
            bucket_correct[bucket] += 1;
        }
    }

    let mut total_error = 0.0;
    let mut measured_buckets = 0;

    for i in 0..buckets {
        if bucket_counts[i] < 2 {
            continue; // Skip buckets with too few samples
        }

        let expected = (i as f64 + 0.5) / buckets as f64;
        let actual = bucket_correct[i] as f64 / bucket_counts[i] as f64;
        total_error += (expected - actual).abs();
        measured_buckets += 1;
    }

    if measured_buckets == 0 {
        return 0.5;
    }

    1.0 - (total_error / measured_buckets as f64)
}

/// Convert Brier score to points (inverted — better score = more points).
/// 1000 points for perfect (0.0), 0 points for random (0.25), negative for worse.
pub fn brier_to_points(brier: f64) -> i64 {
    ((0.25 - brier) * 4000.0) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_brier() {
        let preds = vec![(1.0, true), (0.0, false), (1.0, true)];
        assert!((brier_score(&preds) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_worst_brier() {
        let preds = vec![(1.0, false), (0.0, true)];
        assert!((brier_score(&preds) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_random_brier() {
        let preds = vec![(0.5, true), (0.5, false), (0.5, true), (0.5, false)];
        assert!((brier_score(&preds) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_brier_to_points() {
        assert_eq!(brier_to_points(0.0), 1000);   // perfect
        assert_eq!(brier_to_points(0.25), 0);      // random
        assert_eq!(brier_to_points(0.5), -1000);   // terrible
    }
}
