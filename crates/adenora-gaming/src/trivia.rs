use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriviaQuestion {
    pub id: Uuid,
    pub category: TriviaCategory,
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub difficulty: Difficulty,
    pub points: i64,
    pub time_limit_secs: u32,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriviaCategory {
    Probability,
    Economics,
    Geopolitics,
    BalkanHistory,
    Sports,
    Science,
    Crypto,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn base_points(&self) -> i64 {
        match self {
            Difficulty::Easy => 100,
            Difficulty::Medium => 200,
            Difficulty::Hard => 300,
        }
    }

    pub fn time_limit(&self) -> u32 {
        match self {
            Difficulty::Easy => 30,
            Difficulty::Medium => 20,
            Difficulty::Hard => 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriviaSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub questions: Vec<Uuid>,
    pub current_index: usize,
    pub score: i64,
    pub correct_count: u32,
    pub total_count: u32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriviaAnswer {
    pub question_id: Uuid,
    pub selected_index: usize,
    pub is_correct: bool,
    pub points_earned: i64,
    pub time_taken_ms: u64,
}

/// Score an answer with time bonus.
/// Faster answers earn more points. Max bonus = 50% of base points.
pub fn score_answer(
    question: &TriviaQuestion,
    selected_index: usize,
    time_taken_ms: u64,
) -> TriviaAnswer {
    let is_correct = selected_index == question.correct_index;

    let points_earned = if is_correct {
        let time_limit_ms = question.time_limit_secs as u64 * 1000;
        let time_ratio = if time_taken_ms >= time_limit_ms {
            0.0
        } else {
            1.0 - (time_taken_ms as f64 / time_limit_ms as f64)
        };
        // Base points + up to 50% time bonus
        let bonus = (question.points as f64 * 0.5 * time_ratio) as i64;
        question.points + bonus
    } else {
        0
    };

    TriviaAnswer {
        question_id: question.id,
        selected_index,
        is_correct,
        points_earned,
        time_taken_ms,
    }
}

/// Generate a starter question bank. In production these come from the database.
pub fn seed_questions() -> Vec<TriviaQuestion> {
    vec![
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Probability,
            question: "If you flip a fair coin 10 times and get heads each time, what is the probability of heads on the 11th flip?".into(),
            options: vec!["Less than 50%".into(), "50%".into(), "More than 50%".into(), "It depends".into()],
            correct_index: 1,
            difficulty: Difficulty::Easy,
            points: 100,
            time_limit_secs: 30,
            explanation: Some("Each coin flip is independent. Past results don't affect future probability.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Probability,
            question: "In a prediction market, if a contract trades at 73 cents, what does the market imply?".into(),
            options: vec!["73% probability".into(), "27% probability".into(), "$0.73 profit".into(), "73x leverage".into()],
            correct_index: 0,
            difficulty: Difficulty::Easy,
            points: 100,
            time_limit_secs: 30,
            explanation: Some("Prediction market prices approximate the market's consensus probability.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Economics,
            question: "What currency does Kosovo use?".into(),
            options: vec!["Kosovo Dinar".into(), "Euro".into(), "Serbian Dinar".into(), "Albanian Lek".into()],
            correct_index: 1,
            difficulty: Difficulty::Easy,
            points: 100,
            time_limit_secs: 30,
            explanation: Some("Kosovo unilaterally adopted the Euro, though it is not in the Eurozone.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Geopolitics,
            question: "Which of these countries is NOT a NATO member?".into(),
            options: vec!["Albania".into(), "North Macedonia".into(), "Serbia".into(), "Montenegro".into()],
            correct_index: 2,
            difficulty: Difficulty::Medium,
            points: 200,
            time_limit_secs: 20,
            explanation: Some("Serbia maintains military neutrality and is not a NATO member.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Probability,
            question: "A batch auction collects orders for 500ms then executes. What advantage does this eliminate?".into(),
            options: vec!["Price discovery".into(), "Latency arbitrage".into(), "Market making".into(), "Limit orders".into()],
            correct_index: 1,
            difficulty: Difficulty::Medium,
            points: 200,
            time_limit_secs: 20,
            explanation: Some("Batch auctions ensure all orders in a window are treated equally, eliminating speed advantages.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::BalkanHistory,
            question: "The Ohrid Framework Agreement (2001) primarily addressed ethnic tensions in which country?".into(),
            options: vec!["Kosovo".into(), "Bosnia".into(), "North Macedonia".into(), "Serbia".into()],
            correct_index: 2,
            difficulty: Difficulty::Hard,
            points: 300,
            time_limit_secs: 15,
            explanation: Some("The Ohrid Agreement ended the 2001 insurgency in Macedonia and expanded rights for ethnic Albanians.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Crypto,
            question: "MiCA (Markets in Crypto-Assets) regulation provides what key benefit for licensed platforms?".into(),
            options: vec!["Tax exemption".into(), "EU passporting".into(), "Free mining".into(), "No KYC required".into()],
            correct_index: 1,
            difficulty: Difficulty::Medium,
            points: 200,
            time_limit_secs: 20,
            explanation: Some("MiCA allows a crypto platform licensed in one EU state to operate across all EU member states.".into()),
        },
        TriviaQuestion {
            id: Uuid::new_v4(),
            category: TriviaCategory::Economics,
            question: "In Adenora's fee model, what percentage of trading fees goes to humanitarian projects?".into(),
            options: vec!["20%".into(), "30%".into(), "40%".into(), "50%".into()],
            correct_index: 2,
            difficulty: Difficulty::Easy,
            points: 100,
            time_limit_secs: 30,
            explanation: Some("Adenora splits fees: 40% operations, 40% charity, 20% market creator.".into()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_answer_gets_points() {
        let q = &seed_questions()[0];
        let answer = score_answer(q, q.correct_index, 5000);
        assert!(answer.is_correct);
        assert!(answer.points_earned > 0);
    }

    #[test]
    fn test_wrong_answer_gets_zero() {
        let q = &seed_questions()[0];
        let wrong = (q.correct_index + 1) % q.options.len();
        let answer = score_answer(q, wrong, 5000);
        assert!(!answer.is_correct);
        assert_eq!(answer.points_earned, 0);
    }

    #[test]
    fn test_faster_answer_gets_more_points() {
        let q = &seed_questions()[0];
        let fast = score_answer(q, q.correct_index, 1000);
        let slow = score_answer(q, q.correct_index, 25000);
        assert!(fast.points_earned > slow.points_earned);
    }
}
